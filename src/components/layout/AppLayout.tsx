import { NavLink, Outlet } from "react-router-dom";
import {
  Download,
  Home,
  Library,
  Search,
  Settings,
} from "lucide-react";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/", label: "首页", icon: Home },
  { to: "/search", label: "搜索", icon: Search },
  { to: "/tasks", label: "任务", icon: Download },
  { to: "/library", label: "库", icon: Library },
  { to: "/settings", label: "设置", icon: Settings },
];

export function AppLayout() {
  return (
    <div className="flex h-screen bg-slate-50 text-slate-900">
      <aside className="flex w-56 shrink-0 flex-col border-r border-slate-200 bg-white">
        <div className="border-b border-slate-100 px-4 py-4">
          <div className="text-base font-semibold tracking-tight">Cliprove</div>
          <div className="text-xs text-slate-500">本地视频采集与管理</div>
        </div>
        <nav className="flex flex-1 flex-col gap-1 p-2">
          {navItems.map(({ to, label, icon: Icon }) => (
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
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>
        <div className="border-t border-slate-100 px-4 py-3 text-[11px] text-slate-400">
          Phase 5 · 打包与文档
        </div>
      </aside>
      <main className="min-w-0 flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}
