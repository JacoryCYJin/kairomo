import test from 'node:test';
import assert from 'node:assert/strict';
import { updateFailureKey } from './updateErrors.ts';

test('check errors distinguish timeout, busy service and invalid manifest', () => {
  assert.equal(updateFailureKey(new Error('update_check_timeout'), 'check'), 'updateCheckTimeout');
  assert.equal(updateFailureKey('update_service_busy', 'check'), 'updateServiceBusy');
  assert.equal(updateFailureKey('update_invalid_release', 'check'), 'updateManifestInvalid');
  assert.equal(
    updateFailureKey('update_platform_unavailable', 'check'),
    'updatePlatformUnavailable',
  );
  assert.equal(updateFailureKey('unexpected transport detail', 'check'), 'updateCheckFailed');
});
test('failed installation stages never appear as a successful check or generic network failure', () => {
  assert.equal(updateFailureKey('update_download_failed', 'install'), 'updateDownloadFailed');
  assert.equal(updateFailureKey('update_signature_failed', 'install'), 'updateSignatureFailed');
  assert.equal(updateFailureKey('update_backup_failed', 'install'), 'updateBackupFailed');
  assert.equal(updateFailureKey('update_persist_failed', 'install'), 'updateBackupFailed');
  assert.equal(updateFailureKey('update_install_failed', 'install'), 'updateInstallFailed');
  assert.equal(updateFailureKey(undefined, 'install'), 'updateFailed');
});
