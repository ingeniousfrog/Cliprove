import { useEffect, useState } from "react";
import { ImageOff, Loader2 } from "lucide-react";
import { proxiedCoverSrc } from "@/lib/media";
import { cn } from "@/lib/utils";
import type { Platform } from "@/types";

interface CoverImageProps {
  src?: string | null;
  platform: Platform | string;
  alt?: string;
  className?: string;
  imageClassName?: string;
}

export function CoverImage({
  src,
  platform,
  alt = "",
  className,
  imageClassName,
}: CoverImageProps) {
  const [status, setStatus] = useState<"loading" | "loaded" | "error">("loading");
  const resolved = proxiedCoverSrc(src, platform);

  useEffect(() => {
    setStatus("loading");
  }, [resolved]);

  if (!resolved) {
    return (
      <div
        className={cn(
          "flex items-center justify-center bg-slate-100 text-slate-400",
          className
        )}
      >
        <ImageOff className="h-6 w-6" />
      </div>
    );
  }

  return (
    <div className={cn("relative overflow-hidden bg-slate-100", className)}>
      {status === "loading" ? (
        <div className="absolute inset-0 flex items-center justify-center">
          <Loader2 className="h-5 w-5 animate-spin text-slate-400" />
        </div>
      ) : null}
      {status === "error" ? (
        <div className="absolute inset-0 flex items-center justify-center text-slate-400">
          <ImageOff className="h-6 w-6" />
        </div>
      ) : null}
      <img
        src={resolved}
        alt={alt}
        className={cn(
          "h-full w-full object-cover transition-opacity",
          status === "loaded" ? "opacity-100" : "opacity-0",
          imageClassName
        )}
        onLoad={() => setStatus("loaded")}
        onError={() => setStatus("error")}
      />
    </div>
  );
}
