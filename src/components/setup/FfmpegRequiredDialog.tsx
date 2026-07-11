import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ensureFfmpeg } from "@/lib/tauri";

interface FfmpegRequiredDialogProps {
  open: boolean;
  onClose: () => void;
  onReady?: () => void;
}

const INSTALL_COMMAND = "brew install ffmpeg";

export function FfmpegRequiredDialog({
  open,
  onClose,
  onReady,
}: FfmpegRequiredDialogProps) {
  const [copied, setCopied] = useState(false);

  const ensureMutation = useMutation({
    mutationFn: ensureFfmpeg,
    onSuccess: (status) => {
      if (status.valid) {
        onReady?.();
        onClose();
      }
    },
  });

  if (!open) return null;

  const copyInstallCommand = async () => {
    try {
      await navigator.clipboard.writeText(INSTALL_COMMAND);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-md rounded-xl bg-white shadow-xl">
        <div className="flex items-start justify-between gap-3 border-b border-slate-100 px-4 py-3">
          <div>
            <div className="text-sm font-medium text-slate-900">需要 FFmpeg</div>
            <p className="mt-1 text-xs text-slate-500">
              下载视频需要 FFmpeg 来合并音视频流。
            </p>
          </div>
          <button
            type="button"
            className="rounded-md p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-600"
            onClick={onClose}
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="space-y-4 px-4 py-4 text-sm text-slate-600">
          <p>应用启动时会自动检测 FFmpeg。若未找到，请先安装后点击重新检测。</p>
          <div className="rounded-md border border-slate-100 bg-slate-50 p-3">
            <div className="text-xs text-slate-500">macOS 安装命令</div>
            <code className="mt-1 block text-xs text-slate-800">{INSTALL_COMMAND}</code>
          </div>
          {ensureMutation.data && !ensureMutation.data.valid ? (
            <Badge tone="danger">{ensureMutation.data.message}</Badge>
          ) : null}
          {ensureMutation.data?.valid && ensureMutation.data.resolvedPath ? (
            <Badge tone="success">已就绪：{ensureMutation.data.resolvedPath}</Badge>
          ) : null}
        </div>

        <div className="flex flex-wrap items-center justify-between gap-2 border-t border-slate-100 px-4 py-3">
          <Link
            to="/settings"
            className="text-xs text-slate-500 hover:text-slate-700"
            onClick={onClose}
          >
            前往设置手动指定路径
          </Link>
          <div className="flex gap-2">
            <Button size="sm" variant="secondary" onClick={copyInstallCommand}>
              {copied ? "已复制" : "复制安装命令"}
            </Button>
            <Button
              size="sm"
              loading={ensureMutation.isPending}
              onClick={() => ensureMutation.mutate()}
            >
              重新检测
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
