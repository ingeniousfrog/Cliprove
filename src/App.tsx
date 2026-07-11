import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { AppLayout } from "@/components/layout/AppLayout";
import { OnboardingWizard } from "@/components/onboarding/OnboardingWizard";
import { HomePage } from "@/pages/HomePage";
import { LibraryPage } from "@/pages/LibraryPage";
import { SearchPage } from "@/pages/SearchPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { TasksPage } from "@/pages/TasksPage";
import { getSettings } from "@/lib/tauri";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5_000,
      refetchOnWindowFocus: false,
    },
  },
});

function AppRoutes() {
  const settingsQuery = useQuery({
    queryKey: ["settings"],
    queryFn: getSettings,
  });

  if (settingsQuery.isLoading) {
    return (
      <div className="flex h-screen items-center justify-center text-sm text-slate-500">
        加载中…
      </div>
    );
  }

  if (settingsQuery.data && !settingsQuery.data.onboardingCompleted) {
    return (
      <OnboardingWizard
        settings={settingsQuery.data}
        onComplete={() => {
          queryClient.setQueryData(["settings"], {
            ...settingsQuery.data,
            onboardingCompleted: true,
          });
          void settingsQuery.refetch();
        }}
      />
    );
  }

  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route index element={<HomePage />} />
        <Route path="search" element={<SearchPage />} />
        <Route path="tasks" element={<TasksPage />} />
        <Route path="library" element={<LibraryPage />} />
        <Route path="settings" element={<SettingsPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AppRoutes />
      </BrowserRouter>
    </QueryClientProvider>
  );
}
