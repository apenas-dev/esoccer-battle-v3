export type PageId = 'match' | 'settings' | 'history' | 'help';

interface NavItem {
  id: PageId;
  label: string;
  icon: string;
}

export const NAV_ITEMS: NavItem[] = [
  { id: 'match', label: 'Partida', icon: '⚽' },
  { id: 'settings', label: 'Configurações', icon: '⚙️' },
  { id: 'history', label: 'Histórico', icon: '📊' },
  { id: 'help', label: 'Ajuda', icon: '❓' },
];

interface SidebarProps {
  currentPage: PageId;
  onNavigate: (page: PageId) => void;
}

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="w-56 min-h-screen bg-gray-900 border-r border-gray-800 flex flex-col">
      <div className="p-4 border-b border-gray-800">
        <h1 className="text-lg font-bold text-gray-100">⚽ E-Soccer</h1>
        <p className="text-xs text-gray-500">Battle V5</p>
      </div>
      <nav className="flex-1 p-2 space-y-1">
        {NAV_ITEMS.map((item) => (
          <button
            key={item.id}
            onClick={() => onNavigate(item.id)}
            className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
              currentPage === item.id
                ? 'bg-gray-800 text-gray-100 font-semibold'
                : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'
            }`}
          >
            <span className="mr-2">{item.icon}</span>
            {item.label}
          </button>
        ))}
      </nav>
      <div className="p-4 border-t border-gray-800">
        <p className="text-xs text-gray-600 text-center">E-Soccer Battle © 2025</p>
      </div>
    </aside>
  );
}
