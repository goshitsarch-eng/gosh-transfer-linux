#include <QApplication>
#include <QCheckBox>
#include <QComboBox>
#include <QDesktopServices>
#include <QFileDialog>
#include <QFormLayout>
#include <functional>
#include <QGroupBox>
#include <QHash>
#include <QHeaderView>
#include <QHBoxLayout>
#include <QInputDialog>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QLabel>
#include <QLineEdit>
#include <QDialog>
#include <QListWidget>
#include <QMainWindow>
#include <QMessageBox>
#include <QProgressBar>
#include <QPushButton>
#include <QSpinBox>
#include <QStackedWidget>
#include <QStatusBar>
#include <QTableWidget>
#include <QTimer>
#include <QUrl>
#include <QVBoxLayout>
#include <QVector>

#include "src/qt/bridge.cxxqt.h"

static QJsonObject parse_json_object(const QString &json)
{
    QJsonDocument doc = QJsonDocument::fromJson(json.toUtf8());
    if (!doc.isObject())
    {
        return QJsonObject();
    }
    return doc.object();
}

static QJsonArray parse_json_array(const QString &json)
{
    QJsonDocument doc = QJsonDocument::fromJson(json.toUtf8());
    if (!doc.isArray())
    {
        return QJsonArray();
    }
    return doc.array();
}

static QString json_stringify(const QJsonObject &obj)
{
    return QString::fromUtf8(QJsonDocument(obj).toJson(QJsonDocument::Compact));
}

static QString json_stringify(const QJsonArray &arr)
{
    return QString::fromUtf8(QJsonDocument(arr).toJson(QJsonDocument::Compact));
}

static QJsonValue get_value(const QJsonObject &obj, const QString &key, const QString &alt = QString())
{
    if (obj.contains(key))
    {
        return obj.value(key);
    }
    if (!alt.isEmpty() && obj.contains(alt))
    {
        return obj.value(alt);
    }
    return QJsonValue();
}

class SendPage : public QWidget
{
public:
    explicit SendPage(EngineBridgeQt *engine, QWidget *parent = nullptr)
        : QWidget(parent), engine_(engine)
    {
        auto *layout = new QVBoxLayout(this);

        auto *header = new QLabel("Send Files");
        header->setStyleSheet("font-size: 18px; font-weight: 600;");
        layout->addWidget(header);

        auto *favRow = new QHBoxLayout();
        favorites_ = new QComboBox();
        favorites_->addItem("No favorites saved");
        favRow->addWidget(new QLabel("Favorite"));
        favRow->addWidget(favorites_, 1);
        manage_favorites_ = new QPushButton("Manage");
        favRow->addWidget(manage_favorites_);
        layout->addLayout(favRow);

        auto *destRow = new QHBoxLayout();
        dest_ = new QLineEdit();
        dest_->setPlaceholderText("IP address or hostname");
        test_button_ = new QPushButton("Test");
        add_favorite_ = new QPushButton("Add Favorite");
        destRow->addWidget(new QLabel("Destination"));
        destRow->addWidget(dest_, 1);
        destRow->addWidget(test_button_);
        destRow->addWidget(add_favorite_);
        layout->addLayout(destRow);

        resolution_label_ = new QLabel();
        resolution_label_->setStyleSheet("color: #666;");
        layout->addWidget(resolution_label_);

        auto *pickerRow = new QHBoxLayout();
        browse_files_ = new QPushButton("Pick Files");
        browse_folder_ = new QPushButton("Pick Folder");
        pickerRow->addWidget(browse_files_);
        pickerRow->addWidget(browse_folder_);
        layout->addLayout(pickerRow);

        selected_list_ = new QListWidget();
        layout->addWidget(selected_list_, 1);

        send_button_ = new QPushButton("Send Files");
        send_button_->setEnabled(false);
        layout->addWidget(send_button_);

        resolve_timer_ = new QTimer(this);
        resolve_timer_->setSingleShot(true);
        resolve_timer_->setInterval(300);

        connect(dest_, &QLineEdit::textChanged, this, [this]() {
            resolve_timer_->start();
        });

        connect(resolve_timer_, &QTimer::timeout, this, [this]() {
            const QString address = dest_->text().trimmed();
            if (address.isEmpty())
            {
                resolution_label_->clear();
                return;
            }
            const QString resultJson = engine_->resolve_address(address);
            const QJsonObject obj = parse_json_object(resultJson);
            const bool success = obj.value("success").toBool(false);
            if (success)
            {
                const QJsonArray ips = obj.value("ips").toArray();
                QStringList list;
                for (const auto &ip : ips)
                {
                    list << ip.toString();
                }
                resolution_label_->setText(QString("Resolved: %1").arg(list.join(", ")));
            }
            else
            {
                const QString error = obj.value("error").toString();
                resolution_label_->setText(QString("Resolve failed: %1").arg(error));
            }
        });

        connect(test_button_, &QPushButton::clicked, this, [this]() {
            const QString address = dest_->text().trimmed();
            if (address.isEmpty())
            {
                return;
            }
            const bool ok = engine_->check_peer(address, port_);
            if (!ok)
            {
                QMessageBox::information(this, "Peer Check", "Peer not reachable");
                return;
            }
            const QJsonObject info = parse_json_object(engine_->get_peer_info(address, port_));
            const QString name = info.value("name").toString("Peer");
            QMessageBox::information(this, "Peer Check", QString("Peer is reachable: %1").arg(name));
        });

        connect(add_favorite_, &QPushButton::clicked, this, [this]() {
            const QString address = dest_->text().trimmed();
            if (address.isEmpty())
            {
                return;
            }
            bool ok = false;
            const QString name = QInputDialog::getText(this, "Add Favorite", "Name", QLineEdit::Normal, address, &ok);
            if (!ok || name.isEmpty())
            {
                return;
            }
            engine_->add_favorite(name, address);
            load_favorites();
        });

        connect(browse_files_, &QPushButton::clicked, this, [this]() {
            const QStringList files = QFileDialog::getOpenFileNames(this, "Select Files");
            if (files.isEmpty())
            {
                return;
            }
            selected_directory_.clear();
            selected_files_ = files;
            refresh_selection();
        });

        connect(browse_folder_, &QPushButton::clicked, this, [this]() {
            const QString folder = QFileDialog::getExistingDirectory(this, "Select Folder");
            if (folder.isEmpty())
            {
                return;
            }
            selected_files_.clear();
            selected_directory_ = folder;
            refresh_selection();
        });

        connect(send_button_, &QPushButton::clicked, this, [this]() {
            const QString address = dest_->text().trimmed();
            if (address.isEmpty())
            {
                return;
            }
            if (!selected_directory_.isEmpty())
            {
                engine_->send_directory(address, port_, selected_directory_);
                selected_directory_.clear();
            }
            else if (!selected_files_.isEmpty())
            {
                QJsonArray arr;
                for (const auto &file : selected_files_)
                {
                    arr.append(file);
                }
                engine_->send_files(address, port_, json_stringify(arr));
                selected_files_.clear();
            }
            refresh_selection();
        });

        connect(favorites_, &QComboBox::currentIndexChanged, this, [this](int idx) {
            if (idx < 0 || idx >= favorites_data_.size())
            {
                return;
            }
            const auto fav = favorites_data_[idx];
            dest_->setText(fav.address);
        });

        connect(manage_favorites_, &QPushButton::clicked, this, [this]() {
            show_manage_favorites_dialog();
        });
    }

    void set_port(int port)
    {
        port_ = port;
    }

    void set_receive_only(bool receive_only)
    {
        const bool enabled = !receive_only;
        send_button_->setEnabled(enabled && has_selection());
        browse_files_->setEnabled(enabled);
        browse_folder_->setEnabled(enabled);
    }

    void load_favorites()
    {
        favorites_->clear();
        favorites_data_.clear();
        const QJsonArray favorites = parse_json_array(engine_->list_favorites());
        if (favorites.isEmpty())
        {
            favorites_->addItem("No favorites saved");
            return;
        }

        for (const auto &item : favorites)
        {
            const QJsonObject obj = item.toObject();
            Favorite fav;
            fav.id = obj.value("id").toString();
            fav.name = obj.value("name").toString();
            fav.address = obj.value("address").toString();
            favorites_data_.push_back(fav);
            favorites_->addItem(QString("%1 (%2)").arg(fav.name, fav.address));
        }
    }

private:
    struct Favorite
    {
        QString id;
        QString name;
        QString address;
    };

    void refresh_selection()
    {
        selected_list_->clear();
        if (!selected_directory_.isEmpty())
        {
            selected_list_->addItem(QString("Folder: %1").arg(selected_directory_));
        }
        else
        {
            for (const auto &file : selected_files_)
            {
                selected_list_->addItem(file);
            }
        }
        send_button_->setEnabled(has_selection());
    }

    bool has_selection() const
    {
        return !selected_directory_.isEmpty() || !selected_files_.isEmpty();
    }

    void show_manage_favorites_dialog()
    {
        QDialog dialog(this);
        dialog.setWindowTitle("Manage Favorites");
        dialog.resize(400, 300);

        auto *layout = new QVBoxLayout(&dialog);
        auto *list = new QTableWidget();
        list->setColumnCount(3);
        list->setHorizontalHeaderLabels({"Name", "Address", ""});
        list->horizontalHeader()->setStretchLastSection(true);

        std::function<void()> refresh_table;
        refresh_table = [this, list, &refresh_table]() {
            const QJsonArray favorites = parse_json_array(engine_->list_favorites());
            list->setRowCount(favorites.size());
            for (int i = 0; i < favorites.size(); ++i)
            {
                const QJsonObject obj = favorites[i].toObject();
                list->setItem(i, 0, new QTableWidgetItem(obj.value("name").toString()));
                list->setItem(i, 1, new QTableWidgetItem(obj.value("address").toString()));
                auto *btn = new QPushButton("Delete");
                list->setCellWidget(i, 2, btn);
                const QString id = obj.value("id").toString();
                connect(btn, &QPushButton::clicked, this, [this, &refresh_table, id]() {
                    if (engine_->delete_favorite(id))
                    {
                        refresh_table();
                        load_favorites();
                    }
                });
            }
        };

        refresh_table();

        layout->addWidget(list);
        auto *close = new QPushButton("Close");
        layout->addWidget(close);
        connect(close, &QPushButton::clicked, &dialog, &QDialog::accept);
        dialog.exec();
    }

    EngineBridgeQt *engine_;
    int port_{53317};
    QLineEdit *dest_{nullptr};
    QLabel *resolution_label_{nullptr};
    QComboBox *favorites_{nullptr};
    QPushButton *manage_favorites_{nullptr};
    QPushButton *test_button_{nullptr};
    QPushButton *add_favorite_{nullptr};
    QPushButton *browse_files_{nullptr};
    QPushButton *browse_folder_{nullptr};
    QPushButton *send_button_{nullptr};
    QListWidget *selected_list_{nullptr};
    QTimer *resolve_timer_{nullptr};
    QStringList selected_files_;
    QString selected_directory_;
    QVector<Favorite> favorites_data_;
};

class ReceivePage : public QWidget
{
public:
    explicit ReceivePage(EngineBridgeQt *engine, QWidget *parent = nullptr)
        : QWidget(parent), engine_(engine)
    {
        auto *layout = new QVBoxLayout(this);
        auto *header = new QLabel("Receive Files");
        header->setStyleSheet("font-size: 18px; font-weight: 600;");
        layout->addWidget(header);
        auto *note = new QLabel("Filename conflicts are auto-resolved with \"(n)\" suffixes.");
        note->setStyleSheet("color: #666;");
        layout->addWidget(note);

        auto *ifaceHeader = new QLabel("Local Addresses");
        ifaceHeader->setStyleSheet("font-weight: 600;");
        layout->addWidget(ifaceHeader);

        interface_table_ = new QTableWidget();
        interface_table_->setColumnCount(3);
        interface_table_->setHorizontalHeaderLabels({"Interface", "IP", "Category"});
        interface_table_->horizontalHeader()->setStretchLastSection(true);
        interface_table_->setEditTriggers(QAbstractItemView::NoEditTriggers);
        layout->addWidget(interface_table_);

        auto *batchRow = new QHBoxLayout();
        accept_all_ = new QPushButton("Accept All");
        reject_all_ = new QPushButton("Reject All");
        batchRow->addWidget(accept_all_);
        batchRow->addWidget(reject_all_);
        batchRow->addStretch(1);
        layout->addLayout(batchRow);

        pending_table_ = new QTableWidget();
        pending_table_->setColumnCount(4);
        pending_table_->setHorizontalHeaderLabels({"Sender", "Files", "Size", "Actions"});
        pending_table_->horizontalHeader()->setStretchLastSection(true);
        pending_table_->setSelectionBehavior(QAbstractItemView::SelectRows);
        pending_table_->setEditTriggers(QAbstractItemView::NoEditTriggers);
        layout->addWidget(pending_table_, 1);

        active_table_ = new QTableWidget();
        active_table_->setColumnCount(4);
        active_table_->setHorizontalHeaderLabels({"Transfer", "Progress", "Speed", "Actions"});
        active_table_->horizontalHeader()->setStretchLastSection(true);
        active_table_->setSelectionBehavior(QAbstractItemView::SelectRows);
        active_table_->setEditTriggers(QAbstractItemView::NoEditTriggers);
        layout->addWidget(active_table_, 1);

        connect(accept_all_, &QPushButton::clicked, this, [this]() {
            engine_->accept_all();
        });
        connect(reject_all_, &QPushButton::clicked, this, [this]() {
            engine_->reject_all();
        });
    }

    void set_on_pending_changed(std::function<void()> callback)
    {
        on_pending_changed_ = std::move(callback);
    }

    void load_interfaces(const QJsonObject &settings)
    {
        const QJsonObject filters = settings.value("interfaceFilters").toObject();
        const bool show_wifi = filters.value("showWifi").toBool(true);
        const bool show_ethernet = filters.value("showEthernet").toBool(true);
        const bool show_vpn = filters.value("showVpn").toBool(true);
        const bool show_docker = filters.value("showDocker").toBool(false);
        const bool show_other = filters.value("showOther").toBool(true);

        const QJsonArray interfaces = parse_json_array(engine_->get_interfaces());
        interface_table_->setRowCount(0);
        for (const auto &item : interfaces)
        {
            const QJsonObject iface = item.toObject();
            const QString name = get_value(iface, "name").toString();
            const QString ip = get_value(iface, "ip").toString();
            const bool loopback = get_value(iface, "is_loopback", "isLoopback").toBool(false);
            if (loopback)
            {
                continue;
            }

            QString category = "Other";
            if (name.startsWith("tailscale") || name.startsWith("tun"))
            {
                category = "VPN";
                if (!show_vpn)
                    continue;
            }
            else if (name.startsWith("wl"))
            {
                category = "WiFi";
                if (!show_wifi)
                    continue;
            }
            else if (name.startsWith("en") || name.startsWith("eth"))
            {
                category = "Ethernet";
                if (!show_ethernet)
                    continue;
            }
            else if (name.startsWith("docker") || name.startsWith("br-"))
            {
                category = "Docker";
                if (!show_docker)
                    continue;
            }
            else
            {
                if (!show_other)
                    continue;
            }

            const int row = interface_table_->rowCount();
            interface_table_->insertRow(row);
            interface_table_->setItem(row, 0, new QTableWidgetItem(name));
            interface_table_->setItem(row, 1, new QTableWidgetItem(ip));
            interface_table_->setItem(row, 2, new QTableWidgetItem(category));
        }
    }

    void add_pending(const QJsonObject &transfer)
    {
        const QString id = get_value(transfer, "id").toString();
        if (pending_rows_.contains(id))
        {
            return;
        }
        const int row = pending_table_->rowCount();
        pending_table_->insertRow(row);

        const QString sender = get_value(transfer, "source_ip", "sourceIp").toString();
        const QJsonArray files = get_value(transfer, "files").toArray();
        const QString fileLabel = files.size() == 1 ? files[0].toObject().value("name").toString() : QString("%1 files").arg(files.size());
        const double totalSize = get_value(transfer, "total_size", "totalSize").toDouble();

        pending_table_->setItem(row, 0, new QTableWidgetItem(sender));
        pending_table_->setItem(row, 1, new QTableWidgetItem(fileLabel));
        pending_table_->setItem(row, 2, new QTableWidgetItem(QString::number(totalSize / (1024 * 1024), 'f', 2) + " MB"));

        auto *actions = new QWidget();
        auto *actionLayout = new QHBoxLayout(actions);
        actionLayout->setContentsMargins(0, 0, 0, 0);
        auto *accept = new QPushButton("Accept");
        auto *reject = new QPushButton("Reject");
        actionLayout->addWidget(accept);
        actionLayout->addWidget(reject);
        pending_table_->setCellWidget(row, 3, actions);

        connect(accept, &QPushButton::clicked, this, [this, id]() {
            engine_->accept_transfer(id);
            remove_pending(id);
        });
        connect(reject, &QPushButton::clicked, this, [this, id]() {
            engine_->reject_transfer(id);
            remove_pending(id);
        });

        pending_rows_.insert(id, row);
        if (on_pending_changed_)
        {
            on_pending_changed_();
        }
    }

    void remove_pending(const QString &id)
    {
        if (!pending_rows_.contains(id))
        {
            return;
        }
        const int row = pending_rows_.value(id);
        pending_table_->removeRow(row);
        pending_rows_.remove(id);
        reindex_rows(pending_rows_, pending_table_);
        if (on_pending_changed_)
        {
            on_pending_changed_();
        }
    }

    void add_active_if_missing(const QString &id, const QString &title)
    {
        if (active_rows_.contains(id))
        {
            return;
        }
        const int row = active_table_->rowCount();
        active_table_->insertRow(row);
        active_table_->setItem(row, 0, new QTableWidgetItem(title));

        auto *progress = new QProgressBar();
        progress->setRange(0, 100);
        active_table_->setCellWidget(row, 1, progress);
        active_table_->setItem(row, 2, new QTableWidgetItem("0 B/s"));

        auto *cancel = new QPushButton("Cancel");
        active_table_->setCellWidget(row, 3, cancel);
        connect(cancel, &QPushButton::clicked, this, [this, id]() {
            engine_->cancel_transfer(id);
        });

        active_rows_.insert(id, row);
    }

    void update_progress(const QString &id, qint64 bytes, qint64 total, qint64 speed)
    {
        if (!active_rows_.contains(id))
        {
            return;
        }
        const int row = active_rows_.value(id);
        const int percent = total > 0 ? static_cast<int>((bytes * 100) / total) : 0;
        auto *progress = qobject_cast<QProgressBar *>(active_table_->cellWidget(row, 1));
        if (progress)
        {
            progress->setValue(percent);
        }
        active_table_->item(row, 2)->setText(QString::number(speed / 1024.0, 'f', 1) + " KB/s");
    }

    void mark_complete(const QString &id, const QString &status)
    {
        if (!active_rows_.contains(id))
        {
            return;
        }
        const int row = active_rows_.value(id);
        active_table_->item(row, 0)->setText(status);
        QTimer::singleShot(3000, this, [this, id]() {
            remove_active(id);
        });
    }

    void remove_active(const QString &id)
    {
        if (!active_rows_.contains(id))
        {
            return;
        }
        const int row = active_rows_.value(id);
        active_table_->removeRow(row);
        active_rows_.remove(id);
        reindex_rows(active_rows_, active_table_);
    }

    int pending_count() const
    {
        return pending_rows_.size();
    }

private:
    static void reindex_rows(QHash<QString, int> &map, QTableWidget *table)
    {
        QHash<QString, int> updated;
        for (auto it = map.begin(); it != map.end(); ++it)
        {
            const QString id = it.key();
            const int row = it.value();
            if (row < table->rowCount())
            {
                updated.insert(id, row);
            }
        }
        map.swap(updated);
    }

    EngineBridgeQt *engine_;
    QPushButton *accept_all_{nullptr};
    QPushButton *reject_all_{nullptr};
    QTableWidget *interface_table_{nullptr};
    QTableWidget *pending_table_{nullptr};
    QTableWidget *active_table_{nullptr};
    QHash<QString, int> pending_rows_;
    QHash<QString, int> active_rows_;
    std::function<void()> on_pending_changed_;
};

class TransfersPage : public QWidget
{
public:
    explicit TransfersPage(EngineBridgeQt *engine, QWidget *parent = nullptr)
        : QWidget(parent), engine_(engine)
    {
        auto *layout = new QVBoxLayout(this);
        auto *header = new QLabel("Transfer History");
        header->setStyleSheet("font-size: 18px; font-weight: 600;");
        layout->addWidget(header);

        clear_button_ = new QPushButton("Clear History");
        layout->addWidget(clear_button_);

        table_ = new QTableWidget();
        table_->setColumnCount(5);
        table_->setHorizontalHeaderLabels({"Direction", "Peer", "Files", "Status", "Completed"});
        table_->horizontalHeader()->setStretchLastSection(true);
        table_->setEditTriggers(QAbstractItemView::NoEditTriggers);
        layout->addWidget(table_, 1);

        connect(clear_button_, &QPushButton::clicked, this, [this]() {
            if (engine_->clear_history())
            {
                refresh_history();
            }
        });
    }

    void refresh_history()
    {
        const QJsonArray records = parse_json_array(engine_->list_history());
        table_->setRowCount(records.size());
        for (int i = 0; i < records.size(); ++i)
        {
            const QJsonObject obj = records[i].toObject();
            table_->setItem(i, 0, new QTableWidgetItem(obj.value("direction").toString()));
            table_->setItem(i, 1, new QTableWidgetItem(obj.value("peer_address").toString()));
            table_->setItem(i, 2, new QTableWidgetItem(QString::number(obj.value("files").toArray().size())));
            table_->setItem(i, 3, new QTableWidgetItem(obj.value("status").toString()));
            table_->setItem(i, 4, new QTableWidgetItem(obj.value("completed_at").toString()));
        }
    }

private:
    EngineBridgeQt *engine_;
    QPushButton *clear_button_{nullptr};
    QTableWidget *table_{nullptr};
};

class SettingsPage : public QWidget
{
public:
    explicit SettingsPage(EngineBridgeQt *engine, QWidget *parent = nullptr)
        : QWidget(parent), engine_(engine)
    {
        auto *layout = new QVBoxLayout(this);
        auto *header = new QLabel("Settings");
        header->setStyleSheet("font-size: 18px; font-weight: 600;");
        layout->addWidget(header);

        auto *form = new QFormLayout();
        device_name_ = new QLineEdit();
        form->addRow("Device name", device_name_);

        port_ = new QSpinBox();
        port_->setRange(1, 65535);
        form->addRow("Port", port_);

        download_dir_ = new QLineEdit();
        auto *dirRow = new QHBoxLayout();
        dirRow->addWidget(download_dir_);
        browse_dir_ = new QPushButton("Browse");
        dirRow->addWidget(browse_dir_);
        auto *dirWrap = new QWidget();
        dirWrap->setLayout(dirRow);
        form->addRow("Download dir", dirWrap);

        receive_only_ = new QCheckBox("Receive only");
        notifications_ = new QCheckBox("Enable notifications");
        form->addRow(receive_only_);
        form->addRow(notifications_);

        theme_ = new QComboBox();
        theme_->addItems({"system", "light", "dark"});
        form->addRow("Theme", theme_);

        max_retries_ = new QSpinBox();
        max_retries_->setRange(0, 10);
        retry_delay_ = new QSpinBox();
        retry_delay_->setRange(0, 30000);
        retry_delay_->setSuffix(" ms");
        form->addRow("Max retries", max_retries_);
        form->addRow("Retry delay", retry_delay_);

        bandwidth_limit_ = new QSpinBox();
        bandwidth_limit_->setRange(0, 1024 * 1024 * 1024);
        bandwidth_limit_->setSuffix(" B/s (0 = unlimited)");
        form->addRow("Bandwidth limit", bandwidth_limit_);
        auto *bandwidth_note = new QLabel("Note: engine config supports this field, but throttling may not be enforced yet.");
        bandwidth_note->setStyleSheet("color: #666;");
        bandwidth_note->setWordWrap(true);
        layout->addWidget(bandwidth_note);

        layout->addLayout(form);

        auto *filtersGroup = new QGroupBox("Interface Filters");
        auto *filtersLayout = new QVBoxLayout(filtersGroup);
        show_wifi_ = new QCheckBox("WiFi");
        show_ethernet_ = new QCheckBox("Ethernet");
        show_vpn_ = new QCheckBox("VPN");
        show_docker_ = new QCheckBox("Docker");
        show_other_ = new QCheckBox("Other");
        filtersLayout->addWidget(show_wifi_);
        filtersLayout->addWidget(show_ethernet_);
        filtersLayout->addWidget(show_vpn_);
        filtersLayout->addWidget(show_docker_);
        filtersLayout->addWidget(show_other_);
        layout->addWidget(filtersGroup);

        auto *trustedGroup = new QGroupBox("Trusted Hosts");
        auto *trustedLayout = new QVBoxLayout(trustedGroup);
        trusted_list_ = new QListWidget();
        trustedLayout->addWidget(trusted_list_);
        auto *trustedButtons = new QHBoxLayout();
        add_trusted_ = new QPushButton("Add");
        remove_trusted_ = new QPushButton("Remove");
        trustedButtons->addWidget(add_trusted_);
        trustedButtons->addWidget(remove_trusted_);
        trustedLayout->addLayout(trustedButtons);
        layout->addWidget(trustedGroup);

        save_button_ = new QPushButton("Save Settings");
        layout->addWidget(save_button_);

        connect(browse_dir_, &QPushButton::clicked, this, [this]() {
            const QString dir = QFileDialog::getExistingDirectory(this, "Select Download Directory");
            if (!dir.isEmpty())
            {
                download_dir_->setText(dir);
            }
        });

        connect(add_trusted_, &QPushButton::clicked, this, [this]() {
            bool ok = false;
            const QString host = QInputDialog::getText(this, "Add Trusted Host", "Host", QLineEdit::Normal, QString(), &ok);
            if (ok && !host.isEmpty())
            {
                trusted_list_->addItem(host);
            }
        });

        connect(remove_trusted_, &QPushButton::clicked, this, [this]() {
            auto *item = trusted_list_->currentItem();
            if (item)
            {
                delete item;
            }
        });

        connect(save_button_, &QPushButton::clicked, this, [this]() {
            save_settings();
        });
    }

    void set_on_settings_saved(std::function<void()> callback)
    {
        on_settings_saved_ = std::move(callback);
    }

    void load_settings()
    {
        const QJsonObject settings = parse_json_object(engine_->get_settings());
        device_name_->setText(settings.value("deviceName").toString());
        port_->setValue(settings.value("port").toInt(53317));
        download_dir_->setText(settings.value("downloadDir").toString());
        receive_only_->setChecked(settings.value("receiveOnly").toBool(false));
        notifications_->setChecked(settings.value("notificationsEnabled").toBool(true));
        theme_->setCurrentText(settings.value("theme").toString("system"));
        max_retries_->setValue(settings.value("maxRetries").toInt(3));
        retry_delay_->setValue(settings.value("retryDelayMs").toInt(1000));

        const int bandwidth = settings.value("bandwidthLimitBps").toInt(0);
        bandwidth_limit_->setValue(bandwidth);

        const QJsonObject filters = settings.value("interfaceFilters").toObject();
        show_wifi_->setChecked(filters.value("showWifi").toBool(true));
        show_ethernet_->setChecked(filters.value("showEthernet").toBool(true));
        show_vpn_->setChecked(filters.value("showVpn").toBool(true));
        show_docker_->setChecked(filters.value("showDocker").toBool(false));
        show_other_->setChecked(filters.value("showOther").toBool(true));

        trusted_list_->clear();
        for (const auto &host : settings.value("trustedHosts").toArray())
        {
            trusted_list_->addItem(host.toString());
        }

        last_port_ = port_->value();
    }

    bool receive_only() const
    {
        return receive_only_->isChecked();
    }

private:
    void save_settings()
    {
        QJsonObject settings;
        settings.insert("deviceName", device_name_->text());
        settings.insert("port", port_->value());
        settings.insert("downloadDir", download_dir_->text());
        settings.insert("receiveOnly", receive_only_->isChecked());
        settings.insert("notificationsEnabled", notifications_->isChecked());
        settings.insert("theme", theme_->currentText());
        settings.insert("maxRetries", max_retries_->value());
        settings.insert("retryDelayMs", retry_delay_->value());

        if (bandwidth_limit_->value() > 0)
        {
            settings.insert("bandwidthLimitBps", bandwidth_limit_->value());
        }
        else
        {
            settings.insert("bandwidthLimitBps", QJsonValue::Null);
        }

        QJsonObject filters;
        filters.insert("showWifi", show_wifi_->isChecked());
        filters.insert("showEthernet", show_ethernet_->isChecked());
        filters.insert("showVpn", show_vpn_->isChecked());
        filters.insert("showDocker", show_docker_->isChecked());
        filters.insert("showOther", show_other_->isChecked());
        settings.insert("interfaceFilters", filters);

        QJsonArray trusted;
        for (int i = 0; i < trusted_list_->count(); ++i)
        {
            trusted.append(trusted_list_->item(i)->text());
        }
        settings.insert("trustedHosts", trusted);

        const QString json = json_stringify(settings);
        const bool ok = engine_->save_settings(json);
        if (!ok)
        {
            QMessageBox::warning(this, "Settings", "Failed to save settings");
            return;
        }

        if (port_->value() != last_port_)
        {
            engine_->change_port(port_->value(), true);
            last_port_ = port_->value();
        }

        if (on_settings_saved_)
        {
            on_settings_saved_();
        }
    }

    EngineBridgeQt *engine_;
    QLineEdit *device_name_{nullptr};
    QSpinBox *port_{nullptr};
    QLineEdit *download_dir_{nullptr};
    QPushButton *browse_dir_{nullptr};
    QCheckBox *receive_only_{nullptr};
    QCheckBox *notifications_{nullptr};
    QComboBox *theme_{nullptr};
    QSpinBox *max_retries_{nullptr};
    QSpinBox *retry_delay_{nullptr};
    QSpinBox *bandwidth_limit_{nullptr};

    QCheckBox *show_wifi_{nullptr};
    QCheckBox *show_ethernet_{nullptr};
    QCheckBox *show_vpn_{nullptr};
    QCheckBox *show_docker_{nullptr};
    QCheckBox *show_other_{nullptr};

    QListWidget *trusted_list_{nullptr};
    QPushButton *add_trusted_{nullptr};
    QPushButton *remove_trusted_{nullptr};

    QPushButton *save_button_{nullptr};
    int last_port_{53317};
    std::function<void()> on_settings_saved_;
};

class AboutPage : public QWidget
{
public:
    explicit AboutPage(EngineBridgeQt *engine, QWidget *parent = nullptr)
        : QWidget(parent), engine_(engine)
    {
        auto *layout = new QVBoxLayout(this);
        auto *title = new QLabel("Gosh Transfer");
        title->setStyleSheet("font-size: 20px; font-weight: 700;");
        layout->addWidget(title);

        auto *version = new QLabel(QString("Version %1").arg(engine_->get_version()));
        version->setStyleSheet("color: #666;");
        layout->addWidget(version);

        auto *desc = new QLabel("Explicit peer-to-peer file transfers over LAN, VPN, and Tailscale.");
        desc->setWordWrap(true);
        layout->addWidget(desc);

        auto *links = new QHBoxLayout();
        auto *website = new QPushButton("Website");
        auto *issues = new QPushButton("Issues");
        links->addWidget(website);
        links->addWidget(issues);
        layout->addLayout(links);

        connect(website, &QPushButton::clicked, this, []() {
            QDesktopServices::openUrl(QUrl("https://github.com/goshitsarch-eng/gosh-transfer-linux"));
        });
        connect(issues, &QPushButton::clicked, this, []() {
            QDesktopServices::openUrl(QUrl("https://github.com/goshitsarch-eng/gosh-transfer-linux/issues"));
        });

        layout->addStretch(1);
    }

private:
    EngineBridgeQt *engine_;
};

class MainWindow : public QMainWindow
{
public:
    explicit MainWindow(EngineBridgeQt *engine)
        : engine_(engine)
    {
        setWindowTitle("Gosh Transfer");
        resize(1024, 768);

        auto *central = new QWidget();
        auto *layout = new QHBoxLayout(central);

        nav_ = new QListWidget();
        nav_->addItems({"Send", "Receive", "Transfers", "Settings", "About"});
        nav_->setFixedWidth(200);

        stack_ = new QStackedWidget();
        send_page_ = new SendPage(engine_);
        receive_page_ = new ReceivePage(engine_);
        transfers_page_ = new TransfersPage(engine_);
        settings_page_ = new SettingsPage(engine_);
        about_page_ = new AboutPage(engine_);

        stack_->addWidget(send_page_);
        stack_->addWidget(receive_page_);
        stack_->addWidget(transfers_page_);
        stack_->addWidget(settings_page_);
        stack_->addWidget(about_page_);

        layout->addWidget(nav_);
        layout->addWidget(stack_, 1);
        setCentralWidget(central);

        status_label_ = new QLabel("Starting...");
        statusBar()->addPermanentWidget(status_label_);

        connect(nav_, &QListWidget::currentRowChanged, stack_, &QStackedWidget::setCurrentIndex);
        nav_->setCurrentRow(0);

        connect(engine_, &EngineBridgeQt::engine_event, this, [this](const QString &event_json) {
            handle_engine_event(event_json);
        });
        connect(engine_, &EngineBridgeQt::engine_error, this, [this](const QString &message) {
            handle_engine_error(message);
        });

        receive_page_->set_on_pending_changed([this]() {
            update_receive_badge();
        });

        // initial load
        settings_page_->load_settings();
        send_page_->load_favorites();
        transfers_page_->refresh_history();
        const QJsonObject settings = parse_json_object(engine_->get_settings());
        send_page_->set_port(settings.value("port").toInt(53317));
        send_page_->set_receive_only(settings.value("receiveOnly").toBool(false));
        status_label_->setText(QString("Port %1").arg(settings.value("port").toInt(53317)));
        receive_page_->load_interfaces(settings);

        const QJsonArray pending = parse_json_array(engine_->get_pending_transfers());
        for (const auto &item : pending)
        {
            receive_page_->add_pending(item.toObject());
        }
        update_receive_badge();

        settings_page_->set_on_settings_saved([this]() {
            const QJsonObject settings = parse_json_object(engine_->get_settings());
            send_page_->set_port(settings.value("port").toInt(53317));
            send_page_->set_receive_only(settings.value("receiveOnly").toBool(false));
            status_label_->setText(QString("Port %1").arg(settings.value("port").toInt(53317)));
            receive_page_->load_interfaces(settings);
        });
    }

private:
    void handle_engine_event(const QString &event_json)
    {
        const QJsonObject event = parse_json_object(event_json);
        if (event.contains("TransferRequest"))
        {
            const QJsonObject transfer = event.value("TransferRequest").toObject();
            receive_page_->add_pending(transfer);
            update_receive_badge();
            return;
        }
        if (event.contains("TransferProgress"))
        {
            const QJsonObject progress = event.value("TransferProgress").toObject();
            const QString id = get_value(progress, "transfer_id", "transferId").toString();
            const QString title = get_value(progress, "current_file", "currentFile").toString("Transfer");
            receive_page_->add_active_if_missing(id, title);
            receive_page_->update_progress(id,
                                           static_cast<qint64>(get_value(progress, "bytes_transferred", "bytesTransferred").toDouble()),
                                           static_cast<qint64>(get_value(progress, "total_bytes", "totalBytes").toDouble()),
                                           static_cast<qint64>(get_value(progress, "speed_bps", "speedBps").toDouble()));
            receive_page_->remove_pending(id);
            update_receive_badge();
            return;
        }
        if (event.contains("TransferComplete"))
        {
            const QJsonObject payload = event.value("TransferComplete").toObject();
            const QString id = get_value(payload, "transfer_id", "transferId").toString();
            receive_page_->mark_complete(id, "Complete");
            transfers_page_->refresh_history();
            update_receive_badge();
            return;
        }
        if (event.contains("TransferFailed"))
        {
            const QJsonObject payload = event.value("TransferFailed").toObject();
            const QString id = get_value(payload, "transfer_id", "transferId").toString();
            const QString error = get_value(payload, "error").toString();
            if (!error.isEmpty())
            {
                status_label_->setText(QString("Transfer failed: %1").arg(error));
            }
            receive_page_->mark_complete(id, "Failed");
            transfers_page_->refresh_history();
            update_receive_badge();
            return;
        }
        if (event.contains("TransferRetry"))
        {
            const QJsonObject payload = event.value("TransferRetry").toObject();
            status_label_->setText(QString("Retry %1/%2: %3")
                                        .arg(get_value(payload, "attempt").toInt())
                                        .arg(get_value(payload, "max_attempts", "maxAttempts").toInt())
                                        .arg(get_value(payload, "error").toString()));
            return;
        }
        if (event.contains("ServerStarted"))
        {
            const QJsonObject payload = event.value("ServerStarted").toObject();
            status_label_->setText(QString("Port %1").arg(get_value(payload, "port").toInt()));
            return;
        }
        if (event.contains("PortChanged"))
        {
            const QJsonObject payload = event.value("PortChanged").toObject();
            status_label_->setText(QString("Port %1").arg(get_value(payload, "new_port", "newPort").toInt()));
            return;
        }
    }

    void handle_engine_error(const QString &message)
    {
        QMessageBox::warning(this, "Engine Error", message);
    }

private:
    void update_receive_badge()
    {
        const int count = receive_page_->pending_count();
        QListWidgetItem *item = nav_->item(1);
        if (!item)
        {
            return;
        }
        if (count > 0)
        {
            item->setText(QString("Receive (%1)").arg(count));
        }
        else
        {
            item->setText("Receive");
        }
    }

    EngineBridgeQt *engine_;
    QListWidget *nav_{nullptr};
    QStackedWidget *stack_{nullptr};
    SendPage *send_page_{nullptr};
    ReceivePage *receive_page_{nullptr};
    TransfersPage *transfers_page_{nullptr};
    SettingsPage *settings_page_{nullptr};
    AboutPage *about_page_{nullptr};
    QLabel *status_label_{nullptr};
};

extern "C" int run_app()
{
    int argc = 0;
    char **argv = nullptr;
    QApplication app(argc, argv);

    auto engine = std::make_unique<EngineBridgeQt>();
    if (!engine->initialize())
    {
        return 1;
    }

    engine->start_server();

    MainWindow window(engine.get());
    window.show();

    return app.exec();
}
