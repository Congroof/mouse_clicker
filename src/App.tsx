import { useCallback, useEffect, useRef, useState } from "react";
import { Box, CssBaseline, ThemeProvider, createTheme } from "@mui/material";
import { listen } from "@tauri-apps/api/event";
import * as notification from "@tauri-apps/plugin-notification";
import { ClickSettings } from "./components/ClickSettings";
import type { ClickType, ClickerStatus } from "./components/clicker-types";
import {
  registerHotkey,
  unregisterHotkey,
  configChange,
  startClicker as startClickerCommand,
  stopClicker as stopClickerCommand,
  VK_CODES,
} from "./types/index";

const theme = createTheme({
  palette: {
    primary: { main: "#3b82f6" },
    secondary: { main: "#64748b" },
    background: {
      default: "#ffffff",
      paper: "#ffffff",
    },
    text: {
      primary: "#1f2937",
      secondary: "#6b7280",
    },
  },
  shape: { borderRadius: 6 },
  typography: {
    fontFamily:
      '-apple-system,BlinkMacSystemFont,"Segoe UI","Microsoft YaHei","PingFang SC","Hiragino Sans GB",sans-serif',
    fontSize: 14,
    h6: {
      fontSize: "1.125rem",
    },
    body1: {
      fontSize: "0.875rem",
    },
    body2: {
      fontSize: "0.8125rem",
    },
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: "none",
          fontWeight: 500,
          fontSize: "0.875rem",
          boxShadow: "none",
          "&:hover": {
            boxShadow: "none",
          },
        },
        sizeLarge: {
          padding: "10px 16px",
          fontSize: "0.9375rem",
        },
      },
    },
    MuiTextField: {
      styleOverrides: {
        root: {
          "& .MuiInputBase-root": {
            fontSize: "0.875rem",
          },
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: {
          height: "24px",
          fontSize: "0.75rem",
        },
      },
    },
    MuiTab: {
      styleOverrides: {
        root: {
          textTransform: "none",
          fontSize: "0.875rem",
          minWidth: 100,
          fontWeight: 500,
        },
      },
    },
    MuiTabs: {
      styleOverrides: {
        root: {
          minHeight: 48,
        },
        indicator: {
          height: 3,
        },
      },
    },
  },
});

function App() {
  const [status, setStatus] = useState<ClickerStatus>("idle");
  const [intervalMs, setIntervalMs] = useState(100);
  const [maxClicks, setMaxClicks] = useState<number | null>(null);
  const [clickType, setClickType] = useState<ClickType>("left");
  const [startStopKey, setStartStopKey] = useState("F6");

  const hotkeyIdRef = useRef(1);
  const isHotkeyRegisteredRef = useRef(false);

  // 提取错误信息的辅助函数
  const getErrorMessage = useCallback((err: unknown): string => {
    if (err instanceof Error) {
      return err.message;
    }
    if (typeof err === "string") {
      return err;
    }
    if (err && typeof err === "object") {
      const errObj = err as Record<string, unknown>;

      // 处理 Rust 枚举错误格式 { ErrorVariant: value }
      const keys = Object.keys(errObj);
      if (keys.length > 0) {
        const errorType = keys[0];
        const errorValue = errObj[errorType];

        if (errorType === "DuplicateRegister") {
          return `快捷键 ID ${errorValue} 已被注册`;
        }
        if (errorType === "NotRegistered") {
          return `快捷键 ID ${errorValue} 未注册`;
        }
        if (errorType === "InternalError") {
          return "内部错误";
        }

        // 其他情况：返回键值对
        return `${errorType}: ${errorValue}`;
      }

      // 尝试提取常见的错误字段
      if (errObj.message && typeof errObj.message === "string") {
        return errObj.message;
      }
      if (errObj.error && typeof errObj.error === "string") {
        return errObj.error;
      }

      return JSON.stringify(err);
    }
    return String(err);
  }, []);

  const showNotification = useCallback(async (title: string, body: string) => {
    try {
      const permissionGranted = await notification.isPermissionGranted();
      if (permissionGranted) {
        notification.sendNotification({ title, body });
      } else {
        console.warn("通知权限未授予");
      }
    } catch (error) {
      console.error("发送通知失败:", error);
    }
  }, []);

  // 注册快捷键
  const setupHotkey = useCallback(async () => {
    try {
      const vk = VK_CODES[startStopKey];
      if (!vk) {
        console.error("无效的快捷键:", startStopKey);
        return;
      }

      try {
        await unregisterHotkey(hotkeyIdRef.current);
      } catch (unregErr) {}

      await registerHotkey(hotkeyIdRef.current, 0, vk, clickType);

      isHotkeyRegisteredRef.current = true;
    } catch (err) {
      const errorMsg = `注册快捷键失败: ${getErrorMessage(err)}`;
      showNotification("快捷键错误", errorMsg);
      console.error("setupHotkey 错误:", err);
    }
  }, [startStopKey, clickType, showNotification, getErrorMessage]);

  // 更新配置
  const updateConfig = useCallback(async () => {
    try {
      const times = maxClicks ?? 0;
      const duration = intervalMs;
      await configChange(times, duration);
    } catch (err) {
      const errorMsg = `更新配置失败: ${getErrorMessage(err)}`;
      showNotification("配置错误", errorMsg);
      console.error("updateConfig 错误:", err);
    }
  }, [maxClicks, intervalMs, showNotification, getErrorMessage]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    // 初始化时请求通知权限
    (async () => {
      try {
        let permissionGranted = await notification.isPermissionGranted();
        console.log("当前通知权限:", permissionGranted);

        if (!permissionGranted) {
          const permission = await notification.requestPermission();
          console.log("请求通知权限结果:", permission);
          permissionGranted = permission === "granted";
        }
      } catch (error) {
        console.error("通知初始化失败:", error);
      }
    })();

    listen<boolean>("clicker-status-changed", (event) => {
      const isRunning = event.payload;
      setStatus(isRunning ? "running" : "idle");
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
      if (isHotkeyRegisteredRef.current) {
        unregisterHotkey(hotkeyIdRef.current).catch(console.error);
      }
    };
  }, []);

  useEffect(() => {
    setupHotkey();
  }, [startStopKey, clickType, setupHotkey]);

  useEffect(() => {
    updateConfig();
  }, [maxClicks, intervalMs, updateConfig]);

  const stopClicker = useCallback(async () => {
    try {
      await stopClickerCommand(hotkeyIdRef.current);
    } catch (err) {
      const errorMsg = `停止点击器失败: ${getErrorMessage(err)}`;
      showNotification("操作错误", errorMsg);
      console.error("stopClicker 错误:", err);
    }
  }, [showNotification, getErrorMessage]);

  const startClicker = useCallback(async () => {
    try {
      await startClickerCommand(hotkeyIdRef.current);
    } catch (err) {
      const errorMsg = `启动点击器失败: ${getErrorMessage(err)}`;
      showNotification("操作错误", errorMsg);
      console.error("startClicker 错误:", err);
    }
  }, [showNotification, getErrorMessage]);

  const handleIntervalChange = useCallback((value: number) => {
    setIntervalMs(Math.max(10, Math.min(value, 60 * 60 * 1000)));
  }, []);

  const handleMaxClickChange = useCallback((value: number | null) => {
    if (value === null || value <= 0) {
      setMaxClicks(null);
      return;
    }
    setMaxClicks(Math.max(1, value));
  }, []);

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box
        sx={{
          height: "100vh",
          width: "100vw",
          display: "flex",
          flexDirection: "column",
          bgcolor: "background.paper",
          overflow: "visible !important",
          p: 0.75,
        }}
      >
        <ClickSettings
          interval={intervalMs}
          maxClicks={maxClicks}
          clickType={clickType}
          startStopKey={startStopKey}
          status={status}
          disabled={status !== "idle"}
          onIntervalChange={handleIntervalChange}
          onMaxClicksChange={handleMaxClickChange}
          onClickTypeChange={setClickType}
          onStartStopKeyChange={setStartStopKey}
          onStart={startClicker}
          onStop={stopClicker}
        />
      </Box>
    </ThemeProvider>
  );
}

export default App;
