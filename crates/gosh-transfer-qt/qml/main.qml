// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Qt - Main QML

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Dialogs

ApplicationWindow {
    id: root
    width: 1024
    height: 768
    minimumWidth: 800
    minimumHeight: 600
    visible: true
    title: "Gosh Transfer"

    // Theme colors (dark mode default)
    readonly property color bgBase: "#0d1117"
    readonly property color bgSurface: "#161b22"
    readonly property color bgCard: "#21262d"
    readonly property color borderDefault: "#30363d"
    readonly property color textPrimary: "#e6edf3"
    readonly property color textSecondary: "#8b949e"
    readonly property color primary: "#238636"
    readonly property color primaryHover: "#2ea043"
    readonly property color destructive: "#da3633"
    readonly property color accent: "#58a6ff"

    // Pending transfer count for badge
    property int pendingTransferCount: pendingTransfersModelGlobal.count

    // Global models for transfers (so they can be accessed from sidebar)
    ListModel {
        id: pendingTransfersModelGlobal
    }

    ListModel {
        id: activeTransfersModelGlobal
    }

    // Helper function to format bytes as human-readable string
    function formatSize(bytes) {
        if (bytes >= 1073741824) {
            return (bytes / 1073741824).toFixed(2) + " GB"
        } else if (bytes >= 1048576) {
            return (bytes / 1048576).toFixed(2) + " MB"
        } else if (bytes >= 1024) {
            return (bytes / 1024).toFixed(2) + " KB"
        } else {
            return bytes + " B"
        }
    }

    // Helper function to format speed
    function formatSpeed(bytesPerSec) {
        if (bytesPerSec >= 1048576) {
            return (bytesPerSec / 1048576).toFixed(1) + " MB/s"
        } else if (bytesPerSec >= 1024) {
            return (bytesPerSec / 1024).toFixed(1) + " KB/s"
        } else {
            return bytesPerSec + " B/s"
        }
    }

    // Helper function to get interface icon
    function getInterfaceIcon(ifName) {
        if (ifName.startsWith("wl")) return "📶"  // WiFi
        if (ifName.startsWith("en") || ifName.startsWith("eth")) return "🔌"  // Ethernet
        if (ifName.startsWith("tailscale") || ifName.startsWith("tun")) return "🔒"  // VPN
        if (ifName.startsWith("docker") || ifName.startsWith("br-")) return "🐳"  // Docker
        return "🌐"  // Generic network
    }

    // Helper function to get interface description
    function getInterfaceDescription(ifName) {
        if (ifName.startsWith("wl")) return "WiFi"
        if (ifName.startsWith("en") || ifName.startsWith("eth")) return "Ethernet"
        if (ifName.startsWith("tailscale")) return "Tailscale VPN"
        if (ifName.startsWith("tun")) return "VPN"
        if (ifName.startsWith("docker") || ifName.startsWith("br-")) return "Docker"
        return ifName
    }

    // Device name property (populated by backend)
    property string deviceName: "My Computer"

    color: bgBase

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Sidebar
        Rectangle {
            Layout.preferredWidth: 220
            Layout.fillHeight: true
            color: bgSurface

            ColumnLayout {
                anchors.fill: parent
                spacing: 0

                // Header
                RowLayout {
                    Layout.margins: 16
                    spacing: 12

                    Image {
                        source: "qrc:/icons/logo.png"
                        sourceSize: Qt.size(24, 24)
                    }

                    Text {
                        text: "Gosh Transfer"
                        font.pixelSize: 16
                        font.weight: Font.DemiBold
                        color: textPrimary
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: borderDefault
                }

                // Navigation
                ListView {
                    id: navList
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.topMargin: 8

                    model: ListModel {
                        ListElement { name: "Send"; icon: "upload" }
                        ListElement { name: "Receive"; icon: "download" }
                        ListElement { name: "Transfers"; icon: "history" }
                        ListElement { name: "Settings"; icon: "settings" }
                        ListElement { name: "About"; icon: "info" }
                    }

                    delegate: ItemDelegate {
                        width: navList.width
                        height: 44
                        highlighted: navList.currentIndex === index

                        contentItem: RowLayout {
                            spacing: 12
                            anchors.leftMargin: 16
                            anchors.rightMargin: 16

                            Text {
                                text: model.name
                                color: textPrimary
                                font.pixelSize: 14
                                Layout.fillWidth: true
                            }

                            // Badge for Receive tab (index 1)
                            Rectangle {
                                visible: index === 1 && root.pendingTransferCount > 0
                                width: 20
                                height: 20
                                radius: 10
                                color: destructive

                                Text {
                                    anchors.centerIn: parent
                                    text: root.pendingTransferCount > 9 ? "9+" : root.pendingTransferCount.toString()
                                    color: textPrimary
                                    font.pixelSize: 11
                                    font.weight: Font.DemiBold
                                }
                            }
                        }

                        background: Rectangle {
                            color: highlighted ? Qt.rgba(88/255, 166/255, 255/255, 0.15) : "transparent"
                            Rectangle {
                                visible: highlighted
                                width: 2
                                height: parent.height
                                color: "#58a6ff"
                            }
                        }

                        onClicked: {
                            navList.currentIndex = index
                            contentStack.currentIndex = index
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: borderDefault
                }

                // Server status
                RowLayout {
                    Layout.margins: 16
                    spacing: 8

                    Rectangle {
                        width: 8
                        height: 8
                        radius: 4
                        color: "#3fb950"
                    }

                    Text {
                        text: "Port 53317"
                        font.pixelSize: 12
                        color: textSecondary
                    }
                }
            }
        }

        // Separator
        Rectangle {
            Layout.preferredWidth: 1
            Layout.fillHeight: true
            color: borderDefault
        }

        // Main content
        StackLayout {
            id: contentStack
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: 0

            // ==================== SEND VIEW ====================
            Rectangle {
                color: bgBase

                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 24
                    contentWidth: availableWidth

                    ColumnLayout {
                        width: parent.width
                        spacing: 16

                        // Header
                        Text {
                            text: "Send Files"
                            color: textPrimary
                            font.pixelSize: 28
                            font.weight: Font.Bold
                        }

                        // Favorites model
                        ListModel {
                            id: favoritesModel
                            // Will be populated by backend
                        }

                        // Favorites Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: favoritesContent.implicitHeight + 32

                            ColumnLayout {
                                id: favoritesContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Favorites"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                Text {
                                    text: "Quick access to saved destinations"
                                    color: textSecondary
                                    font.pixelSize: 12
                                }

                                // Favorites dropdown
                                ComboBox {
                                    id: favoritesDropdown
                                    Layout.fillWidth: true
                                    model: favoritesModel.count > 0 ? favoritesModel : ["No favorites saved"]
                                    enabled: favoritesModel.count > 0
                                    textRole: "display"

                                    background: Rectangle {
                                        color: bgSurface
                                        radius: 6
                                        border.color: favoritesDropdown.focus ? accent : borderDefault
                                        border.width: 1
                                    }

                                    contentItem: Text {
                                        text: favoritesDropdown.displayText
                                        color: favoritesDropdown.enabled ? textPrimary : textSecondary
                                        font.pixelSize: 13
                                        leftPadding: 10
                                        verticalAlignment: Text.AlignVCenter
                                    }

                                    onCurrentIndexChanged: {
                                        if (favoritesModel.count > 0 && currentIndex >= 0) {
                                            var item = favoritesModel.get(currentIndex)
                                            if (item && item.address) {
                                                addressField.text = item.address
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Destination Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: destContent.implicitHeight + 32

                            ColumnLayout {
                                id: destContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Destination"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                Text {
                                    text: "Enter the hostname or IP address of the recipient"
                                    color: textSecondary
                                    font.pixelSize: 12
                                }

                                // Address field
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 4

                                    Text {
                                        text: "Address"
                                        color: textSecondary
                                        font.pixelSize: 12
                                    }

                                    TextField {
                                        id: addressField
                                        Layout.fillWidth: true
                                        placeholderText: "e.g., 192.168.1.100 or hostname"
                                        color: textPrimary
                                        placeholderTextColor: textSecondary
                                        background: Rectangle {
                                            color: bgSurface
                                            radius: 6
                                            border.color: addressField.focus ? accent : borderDefault
                                            border.width: 1
                                        }
                                        padding: 10
                                    }
                                }

                                // Save to Favorites row
                                Rectangle {
                                    Layout.fillWidth: true
                                    color: bgSurface
                                    radius: 6
                                    implicitHeight: 48

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: 12
                                        spacing: 8

                                        Text {
                                            text: "Save to Favorites"
                                            color: textPrimary
                                            font.pixelSize: 13
                                            Layout.fillWidth: true
                                        }

                                        Text {
                                            text: "Save this destination for quick access"
                                            color: textSecondary
                                            font.pixelSize: 12
                                        }

                                        Button {
                                            id: saveFavoriteBtn
                                            implicitWidth: 36
                                            implicitHeight: 36
                                            enabled: addressField.text.length > 0

                                            contentItem: Text {
                                                text: "★"
                                                color: saveFavoriteBtn.enabled ? "#f0c000" : textSecondary
                                                font.pixelSize: 18
                                                horizontalAlignment: Text.AlignHCenter
                                                verticalAlignment: Text.AlignVCenter
                                            }

                                            background: Rectangle {
                                                color: saveFavoriteBtn.hovered ? bgCard : "transparent"
                                                radius: 4
                                            }

                                            onClicked: {
                                                saveFavoriteDialog.open()
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Save Favorite Dialog
                        Dialog {
                            id: saveFavoriteDialog
                            title: "Add to Favorites"
                            modal: true
                            anchors.centerIn: parent
                            standardButtons: Dialog.Cancel | Dialog.Ok

                            background: Rectangle {
                                color: bgCard
                                radius: 8
                                border.color: borderDefault
                                border.width: 1
                            }

                            ColumnLayout {
                                spacing: 12

                                Text {
                                    text: "Enter a name for this destination"
                                    color: textSecondary
                                    font.pixelSize: 12
                                }

                                TextField {
                                    id: favoriteNameField
                                    Layout.fillWidth: true
                                    placeholderText: "Name"
                                    text: addressField.text
                                    color: textPrimary
                                    placeholderTextColor: textSecondary
                                    background: Rectangle {
                                        color: bgSurface
                                        radius: 6
                                        border.color: favoriteNameField.focus ? accent : borderDefault
                                        border.width: 1
                                    }
                                    padding: 10
                                }
                            }

                            onAccepted: {
                                if (favoriteNameField.text.length > 0 && addressField.text.length > 0) {
                                    var displayText = favoriteNameField.text + " (" + addressField.text + ")"
                                    favoritesModel.append({
                                        "name": favoriteNameField.text,
                                        "address": addressField.text,
                                        "display": displayText
                                    })
                                    favoriteNameField.text = ""
                                }
                            }

                            onRejected: {
                                favoriteNameField.text = ""
                            }
                        }

                        // Files Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: filesContent.implicitHeight + 32

                            ColumnLayout {
                                id: filesContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Files"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                Text {
                                    text: "Select files to send"
                                    color: textSecondary
                                    font.pixelSize: 12
                                }

                                // Drop zone / file selection
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: 100
                                    color: "transparent"
                                    radius: 8
                                    border.color: borderDefault
                                    border.width: 2

                                    ColumnLayout {
                                        anchors.centerIn: parent
                                        spacing: 8

                                        Text {
                                            text: selectedFiles.count > 0 ?
                                                  selectedFiles.count + " file(s) selected" :
                                                  "No files selected"
                                            color: textSecondary
                                            font.pixelSize: 14
                                            Layout.alignment: Qt.AlignHCenter
                                        }

                                        Button {
                                            text: "Browse Files"
                                            Layout.alignment: Qt.AlignHCenter

                                            contentItem: Text {
                                                text: parent.text
                                                color: textPrimary
                                                font.pixelSize: 13
                                                horizontalAlignment: Text.AlignHCenter
                                            }

                                            background: Rectangle {
                                                color: parent.hovered ? bgSurface : bgCard
                                                radius: 6
                                                border.color: borderDefault
                                                border.width: 1
                                            }

                                            onClicked: fileDialog.open()
                                        }
                                    }
                                }

                                // Selected files list
                                ListView {
                                    id: selectedFiles
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: Math.min(contentHeight, 150)
                                    visible: count > 0
                                    clip: true

                                    model: ListModel { id: filesModel }

                                    delegate: Rectangle {
                                        width: selectedFiles.width
                                        height: 36
                                        color: "transparent"

                                        RowLayout {
                                            anchors.fill: parent
                                            spacing: 8

                                            Text {
                                                text: model.name
                                                color: textPrimary
                                                font.pixelSize: 13
                                                elide: Text.ElideMiddle
                                                Layout.fillWidth: true
                                            }

                                            Button {
                                                text: "x"
                                                implicitWidth: 24
                                                implicitHeight: 24

                                                contentItem: Text {
                                                    text: parent.text
                                                    color: destructive
                                                    font.pixelSize: 14
                                                    horizontalAlignment: Text.AlignHCenter
                                                }

                                                background: Rectangle {
                                                    color: "transparent"
                                                }

                                                onClicked: filesModel.remove(index)
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Send button
                        Button {
                            text: "Send Files"
                            Layout.alignment: Qt.AlignHCenter
                            enabled: addressField.text.length > 0 && filesModel.count > 0

                            contentItem: Text {
                                text: parent.text
                                color: parent.enabled ? textPrimary : textSecondary
                                font.pixelSize: 14
                                font.weight: Font.DemiBold
                                horizontalAlignment: Text.AlignHCenter
                            }

                            background: Rectangle {
                                color: parent.enabled ? (parent.hovered ? primaryHover : primary) : bgCard
                                radius: 20
                                implicitWidth: 140
                                implicitHeight: 40
                            }
                        }

                        Item { Layout.fillHeight: true }
                    }
                }

                FileDialog {
                    id: fileDialog
                    title: "Select files to send"
                    fileMode: FileDialog.OpenFiles
                    onAccepted: {
                        for (var i = 0; i < selectedFiles.length; i++) {
                            var path = selectedFiles[i].toString()
                            var name = path.substring(path.lastIndexOf('/') + 1)
                            filesModel.append({"name": name, "path": path})
                        }
                    }
                }
            }

            // ==================== RECEIVE VIEW ====================
            Rectangle {
                color: bgBase

                // Model for network addresses (populated by backend)
                ListModel {
                    id: networkAddressesModel
                    // Default addresses - will be populated by backend
                    ListElement { ifName: "eth0"; address: "192.168.1.100" }
                    ListElement { ifName: "wlan0"; address: "192.168.1.101" }
                }


                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 24
                    contentWidth: availableWidth

                    ColumnLayout {
                        width: parent.width
                        spacing: 16

                        // Header
                        Text {
                            text: "Receive Files"
                            color: textPrimary
                            font.pixelSize: 28
                            font.weight: Font.Bold
                        }

                        // Your Addresses Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: addressesContent.implicitHeight + 32

                            ColumnLayout {
                                id: addressesContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Your Addresses"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                Text {
                                    text: "Share one of these with the sender"
                                    color: textSecondary
                                    font.pixelSize: 12
                                }

                                // Address list
                                Repeater {
                                    model: networkAddressesModel

                                    Rectangle {
                                        Layout.fillWidth: true
                                        color: bgSurface
                                        radius: 6
                                        implicitHeight: 56

                                        RowLayout {
                                            anchors.fill: parent
                                            anchors.margins: 12
                                            spacing: 12

                                            // Interface icon
                                            Text {
                                                text: getInterfaceIcon(model.ifName)
                                                font.pixelSize: 20
                                            }

                                            ColumnLayout {
                                                Layout.fillWidth: true
                                                spacing: 2

                                                // IP:Port format
                                                Text {
                                                    text: model.address + ":53317"
                                                    color: textPrimary
                                                    font.pixelSize: 13
                                                }

                                                // Interface description
                                                Text {
                                                    text: getInterfaceDescription(model.ifName)
                                                    color: textSecondary
                                                    font.pixelSize: 12
                                                }
                                            }

                                            Button {
                                                text: "Copy"
                                                implicitWidth: 60
                                                implicitHeight: 28

                                                contentItem: Text {
                                                    text: parent.text
                                                    color: textPrimary
                                                    font.pixelSize: 12
                                                    horizontalAlignment: Text.AlignHCenter
                                                }

                                                background: Rectangle {
                                                    color: parent.hovered ? bgCard : "transparent"
                                                    radius: 4
                                                    border.color: borderDefault
                                                    border.width: 1
                                                }

                                                onClicked: {
                                                    // Copy to clipboard with port
                                                    copyHelper.text = model.address + ":53317"
                                                    copyHelper.selectAll()
                                                    copyHelper.copy()
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Hidden text edit for clipboard operations
                        TextEdit {
                            id: copyHelper
                            visible: false
                        }

                        // Server Status Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: serverContent.implicitHeight + 32

                            ColumnLayout {
                                id: serverContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Server Status"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                // Device Name row
                                Rectangle {
                                    Layout.fillWidth: true
                                    color: bgSurface
                                    radius: 6
                                    implicitHeight: 48

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: 12

                                        Text {
                                            text: "Device Name:"
                                            color: textSecondary
                                            font.pixelSize: 13
                                        }

                                        Text {
                                            text: deviceName
                                            color: textPrimary
                                            font.pixelSize: 13
                                        }
                                    }
                                }

                                // Port row
                                Rectangle {
                                    Layout.fillWidth: true
                                    color: bgSurface
                                    radius: 6
                                    implicitHeight: 48

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: 12

                                        Text {
                                            text: "Port:"
                                            color: textSecondary
                                            font.pixelSize: 13
                                        }

                                        Text {
                                            text: "53317"
                                            color: textPrimary
                                            font.pixelSize: 13
                                        }

                                        Item { Layout.fillWidth: true }

                                        Rectangle {
                                            width: 8
                                            height: 8
                                            radius: 4
                                            color: "#3fb950"
                                        }

                                        Text {
                                            text: "Running"
                                            color: "#3fb950"
                                            font.pixelSize: 12
                                        }
                                    }
                                }
                            }
                        }

                        // Pending Transfers Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: pendingContent.implicitHeight + 32

                            ColumnLayout {
                                id: pendingContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Pending Transfers"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                Text {
                                    text: "Incoming transfer requests will appear here"
                                    color: textSecondary
                                    font.pixelSize: 12
                                }

                                // Pending transfers list or empty state
                                Loader {
                                    Layout.fillWidth: true
                                    sourceComponent: pendingTransfersModelGlobal.count > 0 ?
                                        pendingTransfersListComponent : pendingEmptyComponent
                                }

                                Component {
                                    id: pendingEmptyComponent

                                    Rectangle {
                                        width: parent ? parent.width : 200
                                        color: bgSurface
                                        radius: 6
                                        implicitHeight: 60

                                        ColumnLayout {
                                            anchors.centerIn: parent
                                            spacing: 4

                                            Text {
                                                text: "No pending transfers"
                                                color: textPrimary
                                                font.pixelSize: 13
                                                Layout.alignment: Qt.AlignHCenter
                                            }
                                            Text {
                                                text: "Waiting for incoming connections..."
                                                color: textSecondary
                                                font.pixelSize: 12
                                                Layout.alignment: Qt.AlignHCenter
                                            }
                                        }
                                    }
                                }

                                Component {
                                    id: pendingTransfersListComponent

                                    ColumnLayout {
                                        width: parent ? parent.width : 200
                                        spacing: 8

                                        Repeater {
                                            model: pendingTransfersModelGlobal

                                            Rectangle {
                                                Layout.fillWidth: true
                                                color: bgSurface
                                                radius: 6
                                                implicitHeight: pendingItemContent.implicitHeight + 24

                                                ColumnLayout {
                                                    id: pendingItemContent
                                                    anchors.fill: parent
                                                    anchors.margins: 12
                                                    spacing: 8

                                                    RowLayout {
                                                        Layout.fillWidth: true
                                                        spacing: 8

                                                        Text {
                                                            text: model.senderName
                                                            color: textPrimary
                                                            font.pixelSize: 14
                                                            font.weight: Font.DemiBold
                                                        }

                                                        Item { Layout.fillWidth: true }

                                                        Text {
                                                            text: model.fileCount + " file(s)" + (model.totalSize ? " - " + formatSize(model.totalSize) : "")
                                                            color: textSecondary
                                                            font.pixelSize: 12
                                                        }
                                                    }

                                                    RowLayout {
                                                        Layout.fillWidth: true
                                                        spacing: 8

                                                        Button {
                                                            text: "Reject"
                                                            implicitWidth: 80
                                                            implicitHeight: 32

                                                            contentItem: Text {
                                                                text: parent.text
                                                                color: destructive
                                                                font.pixelSize: 13
                                                                horizontalAlignment: Text.AlignHCenter
                                                            }

                                                            background: Rectangle {
                                                                color: parent.hovered ? Qt.rgba(218/255, 54/255, 51/255, 0.1) : "transparent"
                                                                radius: 6
                                                                border.color: destructive
                                                                border.width: 1
                                                            }

                                                            onClicked: {
                                                                // Reject transfer
                                                                pendingTransfersModelGlobal.remove(index)
                                                            }
                                                        }

                                                        Button {
                                                            text: "Accept"
                                                            implicitWidth: 80
                                                            implicitHeight: 32

                                                            contentItem: Text {
                                                                text: parent.text
                                                                color: textPrimary
                                                                font.pixelSize: 13
                                                                horizontalAlignment: Text.AlignHCenter
                                                            }

                                                            background: Rectangle {
                                                                color: parent.hovered ? primaryHover : primary
                                                                radius: 6
                                                            }

                                                            onClicked: {
                                                                // Accept transfer - move to active
                                                                var transfer = pendingTransfersModelGlobal.get(index)
                                                                activeTransfersModelGlobal.append({
                                                                    "transferId": transfer.transferId,
                                                                    "senderName": transfer.senderName,
                                                                    "progress": 0.0,
                                                                    "bytesTransferred": 0,
                                                                    "totalBytes": transfer.totalSize || 0,
                                                                    "speedBps": 0,
                                                                    "isComplete": false,
                                                                    "isFailed": false
                                                                })
                                                                pendingTransfersModelGlobal.remove(index)
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Active Transfers Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: activeContent.implicitHeight + 32

                            ColumnLayout {
                                id: activeContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Active Transfers"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                // Active transfers list or empty state
                                Loader {
                                    Layout.fillWidth: true
                                    sourceComponent: activeTransfersModelGlobal.count > 0 ?
                                        activeTransfersListComponent : activeEmptyComponent
                                }

                                Component {
                                    id: activeEmptyComponent

                                    Rectangle {
                                        width: parent ? parent.width : 200
                                        color: bgSurface
                                        radius: 6
                                        implicitHeight: 48

                                        Text {
                                            anchors.centerIn: parent
                                            text: "No active transfers"
                                            color: textSecondary
                                            font.pixelSize: 13
                                        }
                                    }
                                }

                                Component {
                                    id: activeTransfersListComponent

                                    ColumnLayout {
                                        width: parent ? parent.width : 200
                                        spacing: 8

                                        Repeater {
                                            model: activeTransfersModelGlobal

                                            Rectangle {
                                                id: activeTransferItem
                                                Layout.fillWidth: true
                                                color: bgSurface
                                                radius: 6
                                                implicitHeight: activeItemContent.implicitHeight + 24

                                                // Auto-remove timer for completed transfers
                                                Timer {
                                                    id: autoRemoveTimer
                                                    interval: model.isComplete ? 3000 : (model.isFailed ? 5000 : 0)
                                                    running: model.isComplete || model.isFailed
                                                    onTriggered: {
                                                        activeTransfersModelGlobal.remove(index)
                                                    }
                                                }

                                                ColumnLayout {
                                                    id: activeItemContent
                                                    anchors.fill: parent
                                                    anchors.margins: 12
                                                    spacing: 8

                                                    RowLayout {
                                                        Layout.fillWidth: true

                                                        Text {
                                                            text: model.senderName
                                                            color: textPrimary
                                                            font.pixelSize: 14
                                                        }

                                                        Item { Layout.fillWidth: true }

                                                        // Percentage display
                                                        Text {
                                                            text: Math.round(model.progress * 100) + "%"
                                                            color: model.isComplete ? primary : (model.isFailed ? destructive : accent)
                                                            font.pixelSize: 13
                                                            font.weight: Font.DemiBold
                                                        }
                                                    }

                                                    // Progress bar
                                                    Rectangle {
                                                        Layout.fillWidth: true
                                                        height: 8
                                                        radius: 4
                                                        color: borderDefault

                                                        Rectangle {
                                                            width: parent.width * model.progress
                                                            height: parent.height
                                                            radius: 4
                                                            color: model.isComplete ? primary : (model.isFailed ? destructive : accent)
                                                        }
                                                    }

                                                    // Status row with bytes transferred and speed
                                                    RowLayout {
                                                        Layout.fillWidth: true

                                                        Text {
                                                            text: {
                                                                if (model.isComplete) return "Completed"
                                                                if (model.isFailed) return "Failed"
                                                                var transferred = model.bytesTransferred ? formatSize(model.bytesTransferred) : "0 B"
                                                                var total = model.totalBytes ? formatSize(model.totalBytes) : "0 B"
                                                                return transferred + " / " + total
                                                            }
                                                            color: model.isComplete ? primary : (model.isFailed ? destructive : textSecondary)
                                                            font.pixelSize: 12
                                                        }

                                                        Item { Layout.fillWidth: true }

                                                        // Speed display
                                                        Text {
                                                            visible: !model.isComplete && !model.isFailed && model.speedBps
                                                            text: model.speedBps ? formatSpeed(model.speedBps) : ""
                                                            color: textSecondary
                                                            font.pixelSize: 12
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Item { Layout.fillHeight: true }
                    }
                }
            }

            // ==================== TRANSFERS VIEW ====================
            Rectangle {
                color: bgBase

                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 24
                    contentWidth: availableWidth

                    ColumnLayout {
                        width: parent.width
                        spacing: 16

                        // Header with clear button
                        RowLayout {
                            Layout.fillWidth: true

                            Text {
                                text: "Transfer History"
                                color: textPrimary
                                font.pixelSize: 28
                                font.weight: Font.Bold
                                Layout.fillWidth: true
                            }

                            Button {
                                text: "Clear History"

                                contentItem: Text {
                                    text: parent.text
                                    color: destructive
                                    font.pixelSize: 13
                                    horizontalAlignment: Text.AlignHCenter
                                }

                                background: Rectangle {
                                    color: parent.hovered ? Qt.rgba(218/255, 54/255, 51/255, 0.1) : "transparent"
                                    radius: 6
                                    border.color: destructive
                                    border.width: 1
                                    implicitWidth: 100
                                    implicitHeight: 32
                                }
                            }
                        }

                        // History Card
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            color: bgCard
                            radius: 8

                            ColumnLayout {
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Recent Transfers"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                // Empty state
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.fillHeight: true
                                    color: bgSurface
                                    radius: 6

                                    ColumnLayout {
                                        anchors.centerIn: parent
                                        spacing: 4

                                        Text {
                                            text: "No transfer history"
                                            color: textPrimary
                                            font.pixelSize: 13
                                            Layout.alignment: Qt.AlignHCenter
                                        }
                                        Text {
                                            text: "Completed transfers will appear here"
                                            color: textSecondary
                                            font.pixelSize: 12
                                            Layout.alignment: Qt.AlignHCenter
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ==================== SETTINGS VIEW ====================
            Rectangle {
                color: bgBase

                ScrollView {
                    anchors.fill: parent
                    anchors.margins: 24
                    contentWidth: availableWidth

                    ColumnLayout {
                        width: parent.width
                        spacing: 16

                        // Header
                        Text {
                            text: "Settings"
                            color: textPrimary
                            font.pixelSize: 28
                            font.weight: Font.Bold
                        }

                        // Device Settings Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: deviceContent.implicitHeight + 32

                            ColumnLayout {
                                id: deviceContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Device"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                // Device name
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 4

                                    Text {
                                        text: "Device Name"
                                        color: textSecondary
                                        font.pixelSize: 12
                                    }

                                    TextField {
                                        id: deviceNameField
                                        Layout.fillWidth: true
                                        text: "My Computer"
                                        color: textPrimary
                                        background: Rectangle {
                                            color: bgSurface
                                            radius: 6
                                            border.color: deviceNameField.focus ? accent : borderDefault
                                            border.width: 1
                                        }
                                        padding: 10
                                    }
                                }
                            }
                        }

                        // Transfer Settings Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: transferContent.implicitHeight + 32

                            ColumnLayout {
                                id: transferContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Transfers"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                // Download directory
                                Rectangle {
                                    Layout.fillWidth: true
                                    color: bgSurface
                                    radius: 6
                                    implicitHeight: 56

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: 12

                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            spacing: 2

                                            Text {
                                                text: "Download Directory"
                                                color: textPrimary
                                                font.pixelSize: 13
                                            }
                                            Text {
                                                text: "~/Downloads"
                                                color: textSecondary
                                                font.pixelSize: 12
                                            }
                                        }

                                        Button {
                                            text: "Browse"

                                            contentItem: Text {
                                                text: parent.text
                                                color: textPrimary
                                                font.pixelSize: 12
                                            }

                                            background: Rectangle {
                                                color: parent.hovered ? bgCard : "transparent"
                                                radius: 4
                                                border.color: borderDefault
                                                border.width: 1
                                                implicitWidth: 60
                                                implicitHeight: 28
                                            }
                                        }
                                    }
                                }

                                // Receive only mode
                                Rectangle {
                                    Layout.fillWidth: true
                                    color: bgSurface
                                    radius: 6
                                    implicitHeight: 56

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: 12

                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            spacing: 2

                                            Text {
                                                text: "Receive Only Mode"
                                                color: textPrimary
                                                font.pixelSize: 13
                                            }
                                            Text {
                                                text: "Disable sending files to others"
                                                color: textSecondary
                                                font.pixelSize: 12
                                            }
                                        }

                                        Switch {
                                            id: receiveOnlySwitch
                                        }
                                    }
                                }
                            }
                        }

                        // Appearance Settings Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: appearanceContent.implicitHeight + 32

                            ColumnLayout {
                                id: appearanceContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Appearance"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                // Theme
                                Rectangle {
                                    Layout.fillWidth: true
                                    color: bgSurface
                                    radius: 6
                                    implicitHeight: 56

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: 12

                                        Text {
                                            text: "Theme"
                                            color: textPrimary
                                            font.pixelSize: 13
                                            Layout.fillWidth: true
                                        }

                                        ComboBox {
                                            id: themeCombo
                                            model: ["System", "Light", "Dark"]
                                            currentIndex: 2

                                            background: Rectangle {
                                                color: bgCard
                                                radius: 4
                                                border.color: borderDefault
                                                border.width: 1
                                                implicitWidth: 100
                                            }

                                            contentItem: Text {
                                                text: themeCombo.displayText
                                                color: textPrimary
                                                font.pixelSize: 13
                                                leftPadding: 8
                                                verticalAlignment: Text.AlignVCenter
                                            }
                                        }
                                    }
                                }

                                // Notifications
                                Rectangle {
                                    Layout.fillWidth: true
                                    color: bgSurface
                                    radius: 6
                                    implicitHeight: 56

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.margins: 12

                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            spacing: 2

                                            Text {
                                                text: "Show Notifications"
                                                color: textPrimary
                                                font.pixelSize: 13
                                            }
                                            Text {
                                                text: "Display system notifications for transfers"
                                                color: textSecondary
                                                font.pixelSize: 12
                                            }
                                        }

                                        Switch {
                                            id: notificationsSwitch
                                            checked: true
                                        }
                                    }
                                }
                            }
                        }

                        // Trusted Hosts Model
                        ListModel {
                            id: trustedHostsModel
                            // Will be populated by backend
                        }

                        // Trusted Hosts Card
                        Rectangle {
                            Layout.fillWidth: true
                            color: bgCard
                            radius: 8
                            implicitHeight: trustedContent.implicitHeight + 32

                            ColumnLayout {
                                id: trustedContent
                                anchors.fill: parent
                                anchors.margins: 16
                                spacing: 12

                                Text {
                                    text: "Trusted Hosts"
                                    color: textPrimary
                                    font.pixelSize: 16
                                    font.weight: Font.DemiBold
                                }

                                Text {
                                    text: "Transfers from these hosts are auto-accepted"
                                    color: textSecondary
                                    font.pixelSize: 12
                                }

                                // Trusted hosts list
                                Repeater {
                                    model: trustedHostsModel

                                    Rectangle {
                                        Layout.fillWidth: true
                                        color: bgSurface
                                        radius: 6
                                        implicitHeight: 44

                                        RowLayout {
                                            anchors.fill: parent
                                            anchors.margins: 12
                                            spacing: 8

                                            Text {
                                                text: model.host
                                                color: textPrimary
                                                font.pixelSize: 13
                                                Layout.fillWidth: true
                                            }

                                            Button {
                                                implicitWidth: 32
                                                implicitHeight: 32

                                                contentItem: Text {
                                                    text: "🗑"
                                                    font.pixelSize: 14
                                                    horizontalAlignment: Text.AlignHCenter
                                                    verticalAlignment: Text.AlignVCenter
                                                }

                                                background: Rectangle {
                                                    color: parent.hovered ? Qt.rgba(218/255, 54/255, 51/255, 0.1) : "transparent"
                                                    radius: 4
                                                }

                                                onClicked: {
                                                    trustedHostsModel.remove(index)
                                                }
                                            }
                                        }
                                    }
                                }

                                // Empty state
                                Rectangle {
                                    visible: trustedHostsModel.count === 0
                                    Layout.fillWidth: true
                                    color: bgSurface
                                    radius: 6
                                    implicitHeight: 44

                                    Text {
                                        anchors.centerIn: parent
                                        text: "No trusted hosts configured"
                                        color: textSecondary
                                        font.pixelSize: 12
                                    }
                                }

                                // Add new host row
                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8

                                    TextField {
                                        id: trustedHostField
                                        Layout.fillWidth: true
                                        placeholderText: "Add trusted host..."
                                        color: textPrimary
                                        placeholderTextColor: textSecondary
                                        background: Rectangle {
                                            color: bgSurface
                                            radius: 6
                                            border.color: trustedHostField.focus ? accent : borderDefault
                                            border.width: 1
                                        }
                                        padding: 10

                                        onAccepted: {
                                            if (text.length > 0) {
                                                trustedHostsModel.append({"host": text})
                                                text = ""
                                            }
                                        }
                                    }

                                    Button {
                                        text: "Add"
                                        enabled: trustedHostField.text.length > 0

                                        contentItem: Text {
                                            text: parent.text
                                            color: parent.enabled ? textPrimary : textSecondary
                                            font.pixelSize: 13
                                            horizontalAlignment: Text.AlignHCenter
                                        }

                                        background: Rectangle {
                                            color: parent.enabled ? (parent.hovered ? primaryHover : primary) : bgCard
                                            radius: 6
                                            implicitWidth: 60
                                            implicitHeight: 36
                                        }

                                        onClicked: {
                                            if (trustedHostField.text.length > 0) {
                                                trustedHostsModel.append({"host": trustedHostField.text})
                                                trustedHostField.text = ""
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Save button
                        Button {
                            text: "Save Settings"
                            Layout.alignment: Qt.AlignHCenter

                            contentItem: Text {
                                text: parent.text
                                color: textPrimary
                                font.pixelSize: 14
                                font.weight: Font.DemiBold
                                horizontalAlignment: Text.AlignHCenter
                            }

                            background: Rectangle {
                                color: parent.hovered ? primaryHover : primary
                                radius: 20
                                implicitWidth: 140
                                implicitHeight: 40
                            }
                        }

                        Item { Layout.fillHeight: true }
                    }
                }
            }

            // ==================== ABOUT VIEW ====================
            Rectangle {
                color: bgBase

                ColumnLayout {
                    anchors.centerIn: parent
                    spacing: 16

                    Image {
                        source: "qrc:/icons/logo.png"
                        sourceSize: Qt.size(64, 64)
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Text {
                        text: "Gosh Transfer"
                        color: textPrimary
                        font.pixelSize: 32
                        font.weight: Font.Bold
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Text {
                        text: "Version 2.0.3"
                        color: textSecondary
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Text {
                        text: "A clean, explicit file transfer application."
                        color: textSecondary
                        font.pixelSize: 14
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Item { height: 16 }

                    Rectangle {
                        Layout.preferredWidth: 300
                        Layout.preferredHeight: 1
                        color: borderDefault
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Item { height: 8 }

                    Text {
                        text: "Gosh Contributors"
                        color: textPrimary
                        font.pixelSize: 13
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Text {
                        text: "Licensed under AGPL-3.0"
                        color: textSecondary
                        font.pixelSize: 12
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Item { height: 16 }

                    RowLayout {
                        Layout.alignment: Qt.AlignHCenter
                        spacing: 12

                        Button {
                            text: "Website"

                            contentItem: Text {
                                text: parent.text
                                color: accent
                                font.pixelSize: 13
                                horizontalAlignment: Text.AlignHCenter
                            }

                            background: Rectangle {
                                color: parent.hovered ? Qt.rgba(88/255, 166/255, 255/255, 0.1) : "transparent"
                                radius: 6
                                border.color: accent
                                border.width: 1
                                implicitWidth: 80
                                implicitHeight: 32
                            }

                            onClicked: Qt.openUrlExternally("https://github.com/gosh-sh/gosh-transfer")
                        }

                        Button {
                            text: "Issues"

                            contentItem: Text {
                                text: parent.text
                                color: accent
                                font.pixelSize: 13
                                horizontalAlignment: Text.AlignHCenter
                            }

                            background: Rectangle {
                                color: parent.hovered ? Qt.rgba(88/255, 166/255, 255/255, 0.1) : "transparent"
                                radius: 6
                                border.color: accent
                                border.width: 1
                                implicitWidth: 80
                                implicitHeight: 32
                            }

                            onClicked: Qt.openUrlExternally("https://github.com/gosh-sh/gosh-transfer/issues")
                        }
                    }
                }
            }
        }
    }
}
