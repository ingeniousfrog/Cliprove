import { useQuery } from "@tanstack/react-query";
import { ensureFfmpeg, sidecarHealth, validatePlatformAuth } from "@/lib/tauri";
import { cn } from "@/lib/utils";

interface SystemReadinessBarProps {
  className?: string;
  onLoginClick?: () => void;
}

export function SystemReadinessBar({
  className,
  onLoginClick,
}: SystemReadinessBarProps) {
  const sidecarQuery = useQuery({
    queryKey: ["sidecar-health"],
    queryFn: sidecarHealth,
    retry: false,
    staleTime: 30_000,
  });

  const ffmpegQuery = useQuery({
    queryKey: ["ffmpeg-status"],
    queryFn: ensureFfmpeg,
    staleTime: 60_000,
  });

  const authQuery = useQuery({
    queryKey: ["bilibili-auth-status"],
    queryFn: () => validatePlatformAuth("bilibili"),
    staleTime: 300_000,
  });

  const items = [
    {
      label: "Sidecar",
      ok: sidecarQuery.data?.status === "ok",
      detail: sidecarQuery.data?.status === "ok" ? "运行中" : "未连接",
    },
    {
      label: "FFmpeg",
      ok: ffmpegQuery.data?.valid === true,
      detail: ffmpegQuery.data?.valid ? "已就绪" : "未找到",
    },
    {
      label: "B站",
      ok: authQuery.data?.valid === true,
      detail: authQuery.data?.valid ? "已登录" : "未登录",
      onClick: onLoginClick,
    },
  ];

  return (
    <div className={cn("space-y-1 border-t border-slate-100 px-3 py-3", className)}>
      {items.map((item) => (
        <button
          key={item.label}
          type="button"
          className={cn(
            "flex w-full items-center justify-between rounded-md px-2 py-1 text-left text-xs",
            item.onClick ? "hover:bg-slate-50" : "cursor-default"
          )}
          onClick={item.onClick}
          disabled={!item.onClick}
        >
          <span className="text-slate-500">{item.label}</span>
          <span className={item.ok ? "text-emerald-600" : "text-slate-400"}>
            {item.detail}
          </span>
        </button>
      ))}
    </div>
  );
}
