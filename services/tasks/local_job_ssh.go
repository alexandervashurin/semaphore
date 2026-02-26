package tasks

import (
	"github.com/semaphoreui/semaphore/db"
)

// installSSHKeys устанавливает SSH ключи
func (t *LocalJob) installSSHKeys() (err error) {
	// SSH ключ для инвентаря
	if t.Inventory.SSHKeyID != nil {
		key, err := t.KeyInstaller.GetKey(*t.Inventory.SSHKeyID)
		if err != nil {
			return err
		}

		t.sshKeyInstallation, err = t.KeyInstaller.Install(key, db.AccessKeyRoleGit, t.Logger)
		if err != nil {
			return err
		}
	}

	// Become ключ
	if t.Inventory.BecomeKeyID != nil {
		key, err := t.KeyInstaller.GetKey(*t.Inventory.BecomeKeyID)
		if err != nil {
			return err
		}

		t.becomeKeyInstallation, err = t.KeyInstaller.Install(key, db.AccessKeyRoleAnsibleBecomeUser, t.Logger)
		if err != nil {
			return err
		}
	}

	return
}

// clearSSHKeys очищает SSH ключи
func (t *LocalJob) clearSSHKeys() {
	if t.sshKeyInstallation.SSHAgent != nil {
		t.sshKeyInstallation.SSHAgent.Close() //nolint:errcheck
	}
	if t.becomeKeyInstallation.SSHAgent != nil {
		t.becomeKeyInstallation.SSHAgent.Close() //nolint:errcheck
	}
	for _, installation := range t.vaultFileInstallations {
		if installation.SSHAgent != nil {
			installation.SSHAgent.Close() //nolint:errcheck
		}
	}
}
