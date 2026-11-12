; 小遥账单 Windows NSIS 安装器钩子。
; 对齐 小遥时光：应用为托盘常驻，覆盖安装/卸载前先结束旧进程，
; 避免旧进程握着 xiaoyao-bill-desktop.exe 导致写入或删除失败。

!macro NSIS_HOOK_PREINSTALL
    DetailPrint "正在结束 xiaoyao-bill-desktop.exe..."
    nsExec::Exec '"taskkill" /F /IM xiaoyao-bill-desktop.exe /T'
    Sleep 1000
!macroend

!macro NSIS_HOOK_PREUNINSTALL
    DetailPrint "正在结束 xiaoyao-bill-desktop.exe..."
    nsExec::Exec '"taskkill" /F /IM xiaoyao-bill-desktop.exe /T'
    Sleep 1000
!macroend

; 卸载不删除用户数据目录（%APPDATA%\xiaoyaobill-desktop）。
; 这是产品约定：回收站、快照、审计日志永不自动清理，由用户在应用内手动处理。
