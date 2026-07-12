import type { PlatformAdapter } from "@/types/adapter";
import type {
  AuthStatus,
  DownloadOptions,
  MediaItem,
  ParsedMedia,
  SearchPage,
  SearchQuery,
} from "@/types/models";
import {
  createDownloadSpec,
  parseLink,
  searchMedia,
  validatePlatformAuth,
} from "@/lib/tauri";

function createClientAdapter(id: "douyin" | "bilibili", name: string): PlatformAdapter {
  return {
    id,
    name,
    supportedFilters:
      id === "douyin" ? ["sort", "publish_time"] : ["sort", "media_type"],
    canHandle(input: string) {
      if (id === "douyin") {
        return /douyin\.com|v\.douyin\.com|iesdouyin\.com/i.test(input);
      }
      return /bilibili\.com|b23\.tv/i.test(input) || /^BV[\w]+$/i.test(input.trim());
    },
    parse(input: string) {
      return parseLink(input);
    },
    search(query: SearchQuery, cursor?: string) {
      return searchMedia(id, query, cursor);
    },
    createDownloadSpec(item: MediaItem, options: DownloadOptions) {
      return createDownloadSpec(item, options);
    },
    validateAuth() {
      return validatePlatformAuth(id);
    },
  };
}

export const douyinAdapter = createClientAdapter("douyin", "抖音");
export const bilibiliAdapter = createClientAdapter("bilibili", "Bilibili");

export const adapters: PlatformAdapter[] = [bilibiliAdapter, douyinAdapter];

export function detectAdapter(input: string): PlatformAdapter | undefined {
  return adapters.find((adapter) => adapter.canHandle(input));
}

export async function validateAllAuth(): Promise<AuthStatus[]> {
  return Promise.all(adapters.map((adapter) => adapter.validateAuth()));
}

export type { ParsedMedia, SearchPage };
