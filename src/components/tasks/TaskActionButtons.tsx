import { Button } from "@/components/ui/button";
import type { DownloadTask } from "@/types";

type TaskAction = "pause" | "resume" | "retry" | "cancel" | "delete";

interface TaskActionButtonsProps {
  task: DownloadTask;
  pendingAction?: { taskId: string; action: TaskAction } | null;
  onAction: (taskId: string, action: TaskAction) => void;
  size?: "sm" | "md";
}

const ACTIVE_STATUSES = new Set([
  "pending",
  "parsing",
  "queued",
  "downloading",
  "post_processing",
]);

export function TaskActionButtons({
  task,
  pendingAction,
  onAction,
  size = "sm",
}: TaskActionButtonsProps) {
  const isPending = (action: TaskAction) =>
    pendingAction?.taskId === task.id && pendingAction.action === action;

  const run = (action: TaskAction) => () => onAction(task.id, action);

  return (
    <div className="flex flex-wrap gap-1">
      {task.status === "interrupted" ? (
        <Button
          size={size}
          variant="secondary"
          loading={isPending("resume")}
          onClick={run("resume")}
        >
          恢复
        </Button>
      ) : null}

      {task.status === "failed" ? (
        <Button
          size={size}
          variant="secondary"
          loading={isPending("retry")}
          onClick={run("retry")}
        >
          重试
        </Button>
      ) : null}

      {ACTIVE_STATUSES.has(task.status) ? (
        <Button
          size={size}
          variant="ghost"
          loading={isPending("cancel")}
          onClick={run("cancel")}
        >
          取消
        </Button>
      ) : null}

      {task.status === "interrupted" ? (
        <Button
          size={size}
          variant="ghost"
          loading={isPending("cancel")}
          onClick={run("cancel")}
        >
          放弃
        </Button>
      ) : null}

      <Button
        size={size}
        variant="danger"
        loading={isPending("delete")}
        onClick={run("delete")}
      >
        删除
      </Button>
    </div>
  );
}
