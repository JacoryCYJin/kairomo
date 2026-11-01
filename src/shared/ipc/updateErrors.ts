export function updateFailureKey(error: unknown, stage: 'check' | 'install') {
  const code = error instanceof Error ? error.message : String(error);
  switch (code) {
    case 'update_check_timeout':
      return 'updateCheckTimeout';
    case 'update_service_busy':
      return 'updateServiceBusy';
    case 'update_invalid_release':
      return 'updateManifestInvalid';
    case 'update_platform_unavailable':
      return 'updatePlatformUnavailable';
    case 'update_signature_failed':
      return 'updateSignatureFailed';
    case 'update_download_failed':
      return 'updateDownloadFailed';
    case 'update_backup_failed':
    case 'update_persist_failed':
      return 'updateBackupFailed';
    case 'update_install_failed':
      return 'updateInstallFailed';
    default:
      return stage === 'check' ? 'updateCheckFailed' : 'updateFailed';
  }
}
