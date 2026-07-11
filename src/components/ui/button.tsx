import type { ButtonHTMLAttributes } from "react";
import { Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";

type Variant = "primary" | "secondary" | "ghost" | "danger";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: "sm" | "md";
  loading?: boolean;
}

const variants: Record<Variant, string> = {
  primary:
    "bg-slate-900 text-white hover:bg-slate-800 active:bg-slate-950 disabled:bg-slate-300",
  secondary:
    "bg-white text-slate-700 border border-slate-200 hover:bg-slate-50 active:bg-slate-100",
  ghost:
    "text-slate-600 hover:bg-slate-100 active:bg-slate-200",
  danger:
    "bg-red-600 text-white hover:bg-red-500 active:bg-red-700",
};

export function Button({
  className,
  variant = "primary",
  size = "md",
  loading = false,
  disabled,
  children,
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        "inline-flex items-center justify-center gap-1.5 rounded-md font-medium transition-all duration-100",
        "active:scale-[0.98] disabled:cursor-not-allowed disabled:active:scale-100",
        size === "sm" ? "h-8 px-3 text-xs" : "h-9 px-4 text-sm",
        variants[variant],
        className
      )}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : null}
      {children}
    </button>
  );
}
