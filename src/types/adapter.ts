import type {
  AuthStatus,
  DownloadOptions,
  DownloadSpec,
  MediaItem,
  ParsedMedia,
  Platform,
  SearchFilterKey,
  SearchPage,
  SearchQuery,
} from "./models";

export interface PlatformAdapter {
  id: Platform;
  name: string;
  supportedFilters: SearchFilterKey[];
  canHandle(input: string): boolean;
  parse(input: string): Promise<ParsedMedia>;
  search(query: SearchQuery, cursor?: string): Promise<SearchPage>;
  createDownloadSpec(
    item: MediaItem,
    options: DownloadOptions
  ): Promise<DownloadSpec>;
  validateAuth(): Promise<AuthStatus>;
}
