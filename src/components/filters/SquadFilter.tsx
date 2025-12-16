import { useState, useRef, useEffect } from 'react';
import { Users, ChevronDown } from 'lucide-react';
import clsx from 'clsx';
import { useConfigStore } from '@stores/configStore';
import { useDashboardFilterStore } from '@stores/dashboardFilterStore';

export default function SquadFilter() {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const squads = useConfigStore((state) => state.squads);
  const { filters, setSquad } = useDashboardFilterStore();

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  const handleSelectSquad = (squadId: string | null) => {
    setSquad(squadId);
    setIsOpen(false);
  };

  const selectedSquad = squads.find((s) => s.id === filters.squadId);
  const displayText = selectedSquad?.name || 'All squads';

  // Don't show if user filter is active (mutually exclusive)
  if (filters.userId) {
    return null;
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={clsx(
          'flex items-center gap-2 px-4 py-2 rounded-lg border text-sm font-medium transition-colors',
          isOpen || filters.squadId
            ? 'bg-primary-50 border-primary-300 text-primary-700'
            : 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50'
        )}
      >
        {selectedSquad?.color && (
          <div
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: selectedSquad.color }}
          />
        )}
        {!selectedSquad && <Users size={16} />}
        <span>{displayText}</span>
        <ChevronDown
          size={16}
          className={clsx('transition-transform', isOpen && 'rotate-180')}
        />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 mt-2 w-56 bg-white border border-gray-200 rounded-lg shadow-lg z-50">
          <div className="p-2">
            {/* All squads option */}
            <button
              onClick={() => handleSelectSquad(null)}
              className={clsx(
                'w-full text-left px-3 py-2 rounded-lg text-sm transition-colors',
                !filters.squadId
                  ? 'bg-primary-50 text-primary-700 font-medium'
                  : 'text-gray-700 hover:bg-gray-100'
              )}
            >
              All squads
            </button>

            {/* Squad options */}
            {squads.length === 0 ? (
              <div className="px-3 py-2 text-sm text-gray-500 text-center">
                No squads configured
              </div>
            ) : (
              squads.map((squad) => (
                <button
                  key={squad.id}
                  onClick={() => handleSelectSquad(squad.id)}
                  className={clsx(
                    'w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-colors',
                    filters.squadId === squad.id
                      ? 'bg-primary-50 text-primary-700 font-medium'
                      : 'text-gray-700 hover:bg-gray-100'
                  )}
                >
                  <div
                    className="w-3 h-3 rounded-full"
                    style={{ backgroundColor: squad.color || '#6366f1' }}
                  />
                  <span className="flex-1">{squad.name}</span>
                  <span className="text-xs text-gray-400">
                    {squad.members.length} {squad.members.length === 1 ? 'member' : 'members'}
                  </span>
                </button>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}
