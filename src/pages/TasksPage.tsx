import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { Card, CardBody, CardHeader } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { listTasks, taskAction } from "@/lib/tauri";
import {
  formatDate,
  formatSpeed,
  platformLabel,
  statusLabel,
} from "@/lib/utils";
import type { DownloadProgress } from "@/types";

export function TasksPage() {
  const queryClient = useQueryClient();

  const { data: tasks = [], isLoading } = useQuery({
    queryKey: ["tasks"],
    queryFn: listTasks,
    refetchInterval: 5000,
  });

  useEffect(() => {
    const unlisten = listen<DownloadProgress>("download-progress", () => {
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
    });
    return () => {
      unlisten.then((dispose) => dispose());
    };
  }, [queryClient]);

  const actionMutation = useMutation({
    mutationFn: ({
      taskId,
      action,
    }: {
      taskId: string;
      action: "pause" | "resume" | "retry" | "cancel";
    }) => taskAction(taskId, action),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tasks"] }),
  });

  const interruptedTasks = tasks.filter((task) => task.status === "interrupted");

  return (
    <div className="mx-auto flex max-w-6xl flex-col gap-4 p-6">
      <div>
        <h1 className="text-xl font-semibold">任务中心</h1>
        <p className="mt-1 text-sm text-slate-500">
          查看排队、进行中、已完成与失败任务；支持恢复上次中断的下载。
        </p>
      </div>

      {interruptedTasks.length > 0 ? (
        <Card>
          <CardHeader
            title="中断任务"
            description={`检测到 ${interruptedTasks.length} 个上次未完成的任务`}
          />
          <CardBody className="flex flex-col gap-2">
            {interruptedTasks.map((task) => (
              <div
                key={task.id}
                className="flex items-center justify-between rounded-lg border border-amber-100 bg-amber-50 px-3 py-2"
              >
                <div>
                  <p className="text-sm font-medium">{task.title}</p>
                  <p className="text-xs text-amber-700">
                    {platformLabel(task.platform)} · {task.stage}
                  </p>
                </div>
                <Button
                  size="sm"
                  onClick={() =>
                    actionMutation.mutate({ taskId: task.id, action: "resume" })
                  }
                >
                  恢复下载
                </Button>
              </div>
            ))}
          </CardBody>
        </Card>
      ) : null}

      <Card>
        <CardHeader title="全部任务" description={`共 ${tasks.length} 条`} />
        <CardBody className="overflow-x-auto">
          {isLoading ? (
            <p className="text-sm text-slate-500">加载中…</p>
          ) : tasks.length === 0 ? (
            <p className="text-sm text-slate-500">暂无任务</p>
          ) : (
            <table className="min-w-full text-left text-sm">
              <thead className="border-b border-slate-100 text-xs text-slate-500">
                <tr>
                  <th className="px-2 py-2 font-medium">标题</th>
                  <th className="px-2 py-2 font-medium">平台</th>
                  <th className="px-2 py-2 font-medium">状态</th>
                  <th className="px-2 py-2 font-medium">进度</th>
                  <th className="px-2 py-2 font-medium">速度</th>
                  <th className="px-2 py-2 font-medium">重试</th>
                  <th className="px-2 py-2 font-medium">更新时间</th>
                  <th className="px-2 py-2 font-medium">操作</th>
                </tr>
              </thead>
              <tbody>
                {tasks.map((task) => (
                  <tr key={task.id} className="border-b border-slate-50">
                    <td className="max-w-[220px] truncate px-2 py-2">
                      {task.title}
                    </td>
                    <td className="px-2 py-2">{platformLabel(task.platform)}</td>
                    <td className="px-2 py-2">
                      <Badge
                        tone={
                          task.status === "completed"
                            ? "success"
                            : task.status === "failed"
                              ? "danger"
                              : task.status === "interrupted"
                                ? "warning"
                                : "default"
                        }
                      >
                        {statusLabel(task.status)}
                      </Badge>
                    </td>
                    <td className="px-2 py-2">
                      {Math.round(task.progress * 100)}%
                    </td>
                    <td className="px-2 py-2">{formatSpeed(task.speedBps)}</td>
                    <td className="px-2 py-2">{task.retryCount}</td>
                    <td className="px-2 py-2 text-xs text-slate-500">
                      {formatDate(task.updatedAt)}
                    </td>
                    <td className="px-2 py-2">
                      <div className="flex gap-1">
                        {task.status === "interrupted" ? (
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() =>
                              actionMutation.mutate({
                                taskId: task.id,
                                action: "resume",
                              })
                            }
                          >
                            恢复
                          </Button>
                        ) : null}
                        {task.status === "failed" ? (
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() =>
                              actionMutation.mutate({
                                taskId: task.id,
                                action: "retry",
                              })
                            }
                          >
                            重试
                          </Button>
                        ) : null}
                        {["queued", "downloading"].includes(task.status) ? (
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() =>
                              actionMutation.mutate({
                                taskId: task.id,
                                action: "cancel",
                              })
                            }
                          >
                            取消
                          </Button>
                        ) : null}
                      </div>
                      {task.error ? (
                        <div className="mt-1 text-xs text-red-600">
                          {task.error.message}
                        </div>
                      ) : null}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </CardBody>
      </Card>
    </div>
  );
}
