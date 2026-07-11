import { useState } from "react";
import { Button } from "@/components/ui/button";
import { updateSettings } from "@/lib/tauri";
import type { AppSettings } from "@/types";

interface OnboardingWizardProps {
  settings: AppSettings;
  onComplete: (settings: AppSettings) => void;
}

const STEPS = [
  {
    title: "欢迎使用 Cliprove",
    body: "本地优先的视频采集与管理工具：发现内容、加入下载队列、在本地库中长期管理。",
  },
  {
    title: "快速开始",
    body: "在首页粘贴分享链接即可解析并下载。下载成功的内容会自动进入「库」，无需手动建库。",
  },
  {
    title: "按需配置",
    body: "FFmpeg 会在启动时自动检测；平台登录仅在下载高清或会员内容时需要。这些都可以稍后在设置中管理。",
  },
] as const;

export function OnboardingWizard({ settings, onComplete }: OnboardingWizardProps) {
  const [step, setStep] = useState(0);
  const [saving, setSaving] = useState(false);

  const finish = async () => {
    setSaving(true);
    try {
      const next = await updateSettings({
        ...settings,
        onboardingCompleted: true,
      });
      onComplete(next);
    } finally {
      setSaving(false);
    }
  };

  const current = STEPS[step];
  const isLast = step === STEPS.length - 1;

  return (
    <div className="flex h-screen items-center justify-center bg-slate-50 p-6">
      <div className="w-full max-w-lg rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <div className="mb-1 text-xs text-slate-400">
          步骤 {step + 1} / {STEPS.length}
        </div>
        <h1 className="text-xl font-semibold text-slate-900">{current.title}</h1>
        <p className="mt-3 text-sm leading-6 text-slate-600">{current.body}</p>

        <div className="mt-8 flex items-center justify-between gap-3">
          <Button
            variant="secondary"
            disabled={step === 0 || saving}
            onClick={() => setStep((value) => Math.max(0, value - 1))}
          >
            上一步
          </Button>
          <div className="flex gap-2">
            {!isLast ? (
              <Button variant="secondary" onClick={finish} disabled={saving}>
                跳过
              </Button>
            ) : null}
            {isLast ? (
              <Button loading={saving} onClick={finish}>
                开始使用
              </Button>
            ) : (
              <Button onClick={() => setStep((value) => value + 1)}>下一步</Button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
