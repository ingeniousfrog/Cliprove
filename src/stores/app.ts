import { create } from "zustand";
import type { DownloadTask, ParsedMedia } from "@/types";

interface AppState {
  parsedMedia: ParsedMedia | null;
  setParsedMedia: (media: ParsedMedia | null) => void;
  recentTasks: DownloadTask[];
  setRecentTasks: (tasks: DownloadTask[]) => void;
}

export const useAppStore = create<AppState>((set) => ({
  parsedMedia: null,
  setParsedMedia: (parsedMedia) => set({ parsedMedia }),
  recentTasks: [],
  setRecentTasks: (recentTasks) => set({ recentTasks }),
}));
