import { invoke } from "@tauri-apps/api/core";

export type ClickType = "left" | "right" | "middle";

export interface HotkeyConfig {
  id: number;
  key: string;
  modifiers: string[];
  clickType: ClickType;
  isActive: boolean;
}

export interface ClickConfig {
  times: number;
  duration: number;
}

// Windows Virtual Key Codes
export const VK_CODES: { [key: string]: number } = {
  F1: 0x70,
  F2: 0x71,
  F3: 0x72,
  F4: 0x73,
  F5: 0x74,
  F6: 0x75,
  F7: 0x76,
  F8: 0x77,
  F9: 0x78,
  F10: 0x79,
  F11: 0x7a,
  F12: 0x7b,
};

// Modifier constants
export const MOD_ALT = 0x0001;
export const MOD_CONTROL = 0x0002;
export const MOD_SHIFT = 0x0004;
export const MOD_WIN = 0x0008;

export function modifiersToCode(modifiers: string[]): number {
  let code = 0;
  if (modifiers.includes("Alt")) code |= MOD_ALT;
  if (modifiers.includes("Ctrl")) code |= MOD_CONTROL;
  if (modifiers.includes("Shift")) code |= MOD_SHIFT;
  if (modifiers.includes("Win")) code |= MOD_WIN;
  return code;
}

// Tauri API 调用函数

/**
 * 注册快捷键
 * @param id 快捷键 ID
 * @param modifiers 修饰键组合（如 MOD_CONTROL）
 * @param vk 虚拟键码
 * @param clickType 点击类型
 */
export async function registerHotkey(
  id: number,
  modifiers: number,
  vk: number,
  clickType: ClickType
): Promise<void> {
  await invoke("register_hotkey", {
    id,
    modifiers,
    vk,
    click_type: clickType, // 后端使用 snake_case
  });
}

/**
 * 取消注册快捷键
 * @param id 快捷键 ID
 */
export async function unregisterHotkey(id: number): Promise<void> {
  await invoke("unregister_hotkey", { id });
}

/**
 * 修改点击配置
 * @param times 点击次数（0 表示无限）
 * @param duration 点击间隔（毫秒）
 */
export async function configChange(
  times: number,
  duration: number
): Promise<void> {
  await invoke("config_change", {
    times,
    duration,
  });
}

/**
 * 手动启动点击器
 * @param id 快捷键 ID
 */
export async function startClicker(id: number): Promise<void> {
  await invoke("start_clicker", { id });
}

/**
 * 手动停止点击器
 * @param id 快捷键 ID
 */
export async function stopClicker(id: number): Promise<void> {
  await invoke("stop_clicker", { id });
}