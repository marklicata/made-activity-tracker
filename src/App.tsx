import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from '@stores/authStore';
import Layout from '@components/Layout';
import Dashboard from '@/pages/Dashboard';
import Roadmap from '@/pages/Roadmap';
import Search from '@/pages/Search';
import Settings from '@/pages/Settings';
import Login from '@/pages/Login';
import ProjectDeepDive from '@/pages/ProjectDeepDive';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }
  
  return <>{children}</>;
}

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<Login />} />
        <Route
          path="/"
          element={
            <ProtectedRoute>
              <Layout />
            </ProtectedRoute>
          }
        >
          <Route index element={<Dashboard />} />
          <Route path="roadmap" element={<Roadmap />} />
          <Route path="search" element={<Search />} />
          <Route path="settings" element={<Settings />} />
          <Route path="projects/:owner/:repo" element={<ProjectDeepDive />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
