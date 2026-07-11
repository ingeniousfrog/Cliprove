import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { taskAction } from "@/lib/tauri";

type TaskAction = "pause" | "resume" | "retry" | "cancel" | "delete";

export function useTaskActions() {
  const queryClient = useQueryClient();
  const [pendingAction, setPendingAction] = useState<{
    taskId: string;
    action: TaskAction;
  } | null>(null);

  const mutation = useMutation({
    mutationFn: ({
      taskId,
      action,
    }: {
      taskId: string;
      action: TaskAction;
    }) => taskAction(taskId, action),
    onMutate: ({ taskId, action }) => {
      setPendingAction({ taskId, action });
    },
    onSettled: () => {
      setPendingAction(null);
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
    },
  });

  const runAction = (taskId: string, action: TaskAction) => {
    mutation.mutate({ taskId, action });
  };

  return { pendingAction, runAction };
}
