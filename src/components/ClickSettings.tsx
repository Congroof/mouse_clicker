import {
  Box,
  Button,
  MenuItem,
  Select,
  TextField,
  Typography,
} from "@mui/material";
import { PlayArrow, Stop } from "@mui/icons-material";
import type { ClickType, ClickerStatus } from "./clicker-types";

type Props = {
  interval: number;
  maxClicks: number | null;
  clickType: ClickType;
  startStopKey: string;
  status: ClickerStatus;
  disabled?: boolean;
  onIntervalChange: (value: number) => void;
  onMaxClicksChange: (value: number | null) => void;
  onClickTypeChange: (value: ClickType) => void;
  onStartStopKeyChange: (key: string) => void;
  onStart: () => void;
  onStop: () => void;
};

const HOTKEY_OPTIONS = [
  { value: "F1", label: "F1" },
  { value: "F2", label: "F2" },
  { value: "F3", label: "F3" },
  { value: "F4", label: "F4" },
  { value: "F5", label: "F5" },
  { value: "F6", label: "F6" },
  { value: "F7", label: "F7" },
  { value: "F8", label: "F8" },
  { value: "F9", label: "F9" },
  { value: "F10", label: "F10" },
  { value: "F11", label: "F11" },
  { value: "F12", label: "F12" },
];

const compactMenuProps = {
  disablePortal: false,
  PaperProps: {
    sx: {
      maxHeight: 160,
      "& .MuiList-root": {
        py: 0.5,
      },
      "& .MuiMenuItem-root": {
        fontSize: "0.8125rem",
        minHeight: "auto",
        py: 0.5,
        px: 1.5,
      },
    },
  },
};

export function ClickSettings({
  interval,
  maxClicks,
  clickType,
  startStopKey,
  status,
  disabled = false,
  onIntervalChange,
  onMaxClicksChange,
  onClickTypeChange,
  onStartStopKeyChange,
  onStart,
  onStop,
}: Props) {
  const isIdle = status === "idle";

  return (
    <Box
      sx={{
        display: "grid",
        gridTemplateColumns: "100px 1fr",
        gap: 0.5,
        rowGap: 0.5,
        alignItems: "center",
      }}
    >
      {/* 间隔时间 */}
      <Typography
        variant="body2"
        sx={{
          color: "text.secondary",
          whiteSpace: "nowrap",
          fontSize: "0.8125rem",
        }}
      >
        间隔(ms)
      </Typography>
      <TextField
        type="number"
        value={interval}
        disabled={disabled}
        onChange={(event) => onIntervalChange(Number(event.target.value))}
        inputProps={{ min: 1, max: 60 * 60 * 1000 }}
        size="small"
      />

      {/* 点击次数 */}
      <Typography
        variant="body2"
        sx={{
          color: "text.secondary",
          whiteSpace: "nowrap",
          fontSize: "0.8125rem",
        }}
      >
        点击次数
      </Typography>
      <TextField
        type="number"
        value={maxClicks ?? 0}
        disabled={disabled}
        placeholder="0: 无限制"
        onChange={(event) => {
          const next = Number(event.target.value);
          if (Number.isNaN(next)) {
            onMaxClicksChange(null);
            return;
          }
          onMaxClicksChange(next <= 0 ? null : next);
        }}
        inputProps={{ min: 0 }}
        size="small"
      />

      {/* 点击类型 */}
      <Typography
        variant="body2"
        sx={{
          color: "text.secondary",
          whiteSpace: "nowrap",
          fontSize: "0.8125rem",
        }}
      >
        点击类型
      </Typography>
      <Select
        value={clickType}
        onChange={(event) => onClickTypeChange(event.target.value as ClickType)}
        disabled={disabled}
        size="small"
        MenuProps={compactMenuProps}
      >
        <MenuItem value="left">左键</MenuItem>
        <MenuItem value="right">右键</MenuItem>
        <MenuItem value="middle">中键</MenuItem>
      </Select>

      {/* 快捷键 */}
      <Typography
        variant="body2"
        sx={{
          color: "text.secondary",
          whiteSpace: "nowrap",
          fontSize: "0.8125rem",
        }}
      >
        快捷键
      </Typography>
      <Box sx={{ display: "flex", gap: 0.5 }}>
        <Select
          value={startStopKey}
          onChange={(event) => onStartStopKeyChange(event.target.value)}
          size="small"
          disabled={disabled}
          sx={{ minWidth: 70 }}
          MenuProps={compactMenuProps}
        >
          {HOTKEY_OPTIONS.map((option) => (
            <MenuItem key={option.value} value={option.value}>
              {option.label}
            </MenuItem>
          ))}
        </Select>
        <Button
          onClick={isIdle ? onStart : onStop}
          variant="contained"
          color={isIdle ? "primary" : "error"}
          size="medium"
          startIcon={isIdle ? <PlayArrow /> : <Stop />}
          sx={{ flex: 1, minWidth: 80 }}
        >
          {isIdle ? "开始" : "停止"}
        </Button>
      </Box>
    </Box>
  );
}
