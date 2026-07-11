import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Card, CardBody, CardHeader } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { listLibrary } from "@/lib/tauri";
import { formatDate, formatDuration, platformLabel } from "@/lib/utils";

export function LibraryPage() {
  const [query, setQuery] = useState("");

  const { data: items = [], isLoading } = useQuery({
    queryKey: ["library", query],
    queryFn: () => listLibrary(query || undefined),
  });

  return (
    <div className="mx-auto flex max-w-6xl flex-col gap-4 p-6">
      <div>
        <h1 className="text-xl font-semibold">本地库</h1>
        <p className="mt-1 text-sm text-slate-500">
          浏览已下载内容，支持按标题、作者与 ID 搜索。
        </p>
      </div>

      <Card>
        <CardBody>
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="搜索标题、作者或平台 ID…"
          />
        </CardBody>
      </Card>

      <Card>
        <CardHeader title="库条目" description={`共 ${items.length} 条`} />
        <CardBody className="overflow-x-auto">
          {isLoading ? (
            <p className="text-sm text-slate-500">加载中…</p>
          ) : items.length === 0 ? (
            <p className="text-sm text-slate-500">库中暂无内容</p>
          ) : (
            <table className="min-w-full text-left text-sm">
              <thead className="border-b border-slate-100 text-xs text-slate-500">
                <tr>
                  <th className="px-2 py-2 font-medium">标题</th>
                  <th className="px-2 py-2 font-medium">平台</th>
                  <th className="px-2 py-2 font-medium">作者</th>
                  <th className="px-2 py-2 font-medium">时长</th>
                  <th className="px-2 py-2 font-medium">来源关键词</th>
                  <th className="px-2 py-2 font-medium">下载时间</th>
                </tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr key={item.id} className="border-b border-slate-50">
                    <td className="max-w-[260px] truncate px-2 py-2">
                      {item.title}
                    </td>
                    <td className="px-2 py-2">
                      <Badge>{platformLabel(item.platform)}</Badge>
                    </td>
                    <td className="px-2 py-2">{item.authorName}</td>
                    <td className="px-2 py-2">
                      {formatDuration(item.durationSec)}
                    </td>
                    <td className="px-2 py-2 text-slate-500">
                      {item.searchKeyword || "—"}
                    </td>
                    <td className="px-2 py-2 text-xs text-slate-500">
                      {formatDate(item.createdAt)}
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
