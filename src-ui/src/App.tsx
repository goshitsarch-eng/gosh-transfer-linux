import { useEffect } from 'react';
import { useAppStore } from './store';
import { Navigation } from './components/Navigation';
import { SendPage } from './pages/SendPage';
import { ReceivePage } from './pages/ReceivePage';
import { TransfersPage } from './pages/TransfersPage';
import { SettingsPage } from './pages/SettingsPage';
import { AboutPage } from './pages/AboutPage';

function App() {
  const { currentPage, settings, initializeEventListener } = useAppStore();

  useEffect(() => {
    initializeEventListener();
  }, [initializeEventListener]);

  useEffect(() => {
    // Apply theme
    if (settings?.theme === 'dark') {
      document.documentElement.classList.add('dark');
    } else if (settings?.theme === 'light') {
      document.documentElement.classList.remove('dark');
    } else {
      // System preference
      if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
    }
  }, [settings?.theme]);

  const renderPage = () => {
    switch (currentPage) {
      case 'send':
        return <SendPage />;
      case 'receive':
        return <ReceivePage />;
      case 'transfers':
        return <TransfersPage />;
      case 'settings':
        return <SettingsPage />;
      case 'about':
        return <AboutPage />;
    }
  };

  return (
    <div className="h-full flex flex-col bg-gray-50 dark:bg-gray-900">
      <Navigation />
      <main className="flex-1 overflow-auto">{renderPage()}</main>
    </div>
  );
}

export default App;
