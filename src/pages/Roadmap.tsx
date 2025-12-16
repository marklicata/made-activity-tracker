import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Calendar, Target, ChevronRight, Loader2 } from 'lucide-react';

interface Milestone {
  id: string;
  title: string;
  description: string | null;
  due_date: string | null;
  repo: string;
  open_issues: number;
  closed_issues: number;
  state: 'open' | 'closed';
}

interface CycleGroup {
  title: string;
  due_date: string | null;
  milestones: Milestone[];
  total_open: number;
  total_closed: number;
}

export default function Roadmap() {
  const [cycles, setCycles] = useState<CycleGroup[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadRoadmap();
  }, []);

  const loadRoadmap = async () => {
    try {
      const data = await invoke<CycleGroup[]>('get_roadmap');
      setCycles(data);
    } catch (error) {
      console.error('Failed to load roadmap:', error);
    } finally {
      setLoading(false);
    }
  };

  const getProgress = (open: number, closed: number) => {
    const total = open + closed;
    if (total === 0) return 0;
    return Math.round((closed / total) * 100);
  };

  if (loading) {
    return (
      <div className="p-8 flex items-center justify-center h-full">
        <div className="flex items-center gap-2 text-gray-400">
          <Loader2 className="animate-spin" size={20} />
          Loading roadmap...
        </div>
      </div>
    );
  }

  return (
    <div className="p-8">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold text-gray-900">Roadmap</h1>
        <p className="text-gray-500">Upcoming features organized by cycles</p>
      </div>

      {/* Cycles */}
      <div className="space-y-6">
        {cycles.map((cycle) => {
          const progress = getProgress(cycle.total_open, cycle.total_closed);
          
          return (
            <div key={cycle.title} className="bg-white rounded-xl shadow-sm border border-gray-200">
              {/* Cycle Header */}
              <div className="p-6 border-b border-gray-100">
                <div className="flex items-start justify-between">
                  <div>
                    <h2 className="text-lg font-semibold text-gray-900 flex items-center gap-2">
                      <Target className="text-blue-600" size={20} />
                      {cycle.title}
                    </h2>
                    {cycle.due_date && (
                      <p className="text-sm text-gray-500 flex items-center gap-1 mt-1">
                        <Calendar size={14} />
                        Due {new Date(cycle.due_date).toLocaleDateString()}
                      </p>
                    )}
                  </div>
                  <div className="text-right">
                    <p className="text-2xl font-bold text-gray-900">{progress}%</p>
                    <p className="text-xs text-gray-500">
                      {cycle.total_closed} / {cycle.total_open + cycle.total_closed} complete
                    </p>
                  </div>
                </div>
                
                {/* Progress Bar */}
                <div className="mt-4 h-2 bg-gray-100 rounded-full overflow-hidden">
                  <div 
                    className="h-full bg-blue-500 transition-all duration-500"
                    style={{ width: `${progress}%` }}
                  />
                </div>
              </div>

              {/* Milestones by Repo */}
              <div className="divide-y divide-gray-100">
                {cycle.milestones.map((milestone) => {
                  const milestoneProgress = getProgress(milestone.open_issues, milestone.closed_issues);
                  
                  return (
                    <div key={milestone.id} className="p-4 hover:bg-gray-50 transition-colors">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                          <ChevronRight size={16} className="text-gray-400" />
                          <div>
                            <p className="font-medium text-gray-800">{milestone.repo}</p>
                            {milestone.description && (
                              <p className="text-sm text-gray-500">{milestone.description}</p>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-4">
                          <div className="w-24 h-1.5 bg-gray-100 rounded-full overflow-hidden">
                            <div 
                              className="h-full bg-blue-400"
                              style={{ width: `${milestoneProgress}%` }}
                            />
                          </div>
                          <span className="text-sm text-gray-500 w-20 text-right">
                            {milestone.closed_issues}/{milestone.open_issues + milestone.closed_issues}
                          </span>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>

      {cycles.length === 0 && (
        <div className="text-center py-12 text-gray-500">
          No milestones found. Make sure your repositories use milestones for cycles.
        </div>
      )}
    </div>
  );
}
