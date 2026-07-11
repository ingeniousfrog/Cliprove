import { useState } from "react";
import { NavLink, Outlet } from "react-router-dom";
import {
  Download,
  Home,
  Library,
  Search,
  Settings,
} from "lucide-react";
import { PlatformAuthDialog } from "@/components/setup/PlatformAuthDialog";
import { SystemReadinessBar } from "@/components/setup/SystemReadinessBar";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/", label: "首页", subtitle: null, icon: Home },
  { to: "/search", label: "搜索", subtitle: null, icon: Search },
  { to: "/tasks", label: "任务", subtitle: null, icon: Download },
  { to: "/library", label: "库", subtitle: "已下载内容", icon: Library },
  { to: "/settings", label: "设置", subtitle: null, icon: Settings },
];

export function AppLayout() {
  const [authDialogOpen, setAuthDialogOpen] = useState(false);

  return (
    <div className="flex h-screen bg-slate-50 text-slate-900">
      <aside className="flex w-56 shrink-0 flex-col border-r border-slate-200 bg-white">
        <div className="border-b border-slate-100 px-4 py-4">
          <div className="text-base font-semibold tracking-tight">Cliprove</div>
          <div className="text-xs text-slate-500">本地视频采集与管理</div>
        </div>
        <nav className="flex flex-1 flex-col gap-1 p-2">
          {navItems.map(({ to, label, subtitle, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
                  isActive
                    ? "bg-slate-900 text-white"
                    : "text-slate-600 hover:bg-slate-100"
                )
              }
            >
              <Icon className="h-4 w-4 shrink-0" />
              <span className="min-w-0">
                <span className="block">{label}</span>
                {subtitle ? (
                  <span className="block text-[10px] opacity-70">{subtitle}</span>
                ) : null}
              </span>
            </NavLink>
          ))}
        </nav>
        <SystemReadinessBar onLoginClick={() => setAuthDialogOpen(true)} />
      </aside>
      <main className="min-w-0 flex-1 overflow-auto">
        <Outlet />
      </main>
      <PlatformAuthDialog
        open={authDialogOpen}
        platform="bilibili"
        onClose={() => setAuthDialogOpen(false)}
      />
    </div>
  );
}
