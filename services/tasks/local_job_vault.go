package tasks

import (
	"fmt"
	"os"
	"path"

	"github.com/semaphoreui/semaphore/db"
	"github.com/semaphoreui/semaphore/db_lib"
	"github.com/semaphoreui/semaphore/util"
)

// installVaultKeyFiles устанавливает файлы ключей Vault
func (t *LocalJob) installVaultKeyFiles() (err error) {
	t.vaultFileInstallations = make(map[string]ssh.AccessKeyInstallation)

	for _, vault := range t.Inventory.Vaults {
		if vault.VaultKeyID == nil {
			continue
		}

		key, err := t.KeyInstaller.GetKey(*vault.VaultKeyID)
		if err != nil {
			return err
		}

		installation, err := t.KeyInstaller.Install(key, db.AccessKeyRoleAnsiblePasswordVault, t.Logger)
		if err != nil {
			return err
		}

		t.vaultFileInstallations[vault.Name] = installation

		if installation.Password != "" {
			// Создаём временный файл для пароля Vault
			tmpDir := t.Task.GetTmpDir()
			vaultPasswordFile := path.Join(tmpDir, fmt.Sprintf("vault_%s_password", vault.Name))

			err = os.WriteFile(vaultPasswordFile, []byte(installation.Password), 0600)
			if err != nil {
				return err
			}
		}
	}

	return
}

// clearVaultKeyFiles очищает файлы ключей Vault
func (t *LocalJob) clearVaultKeyFiles() {
	for name, installation := range t.vaultFileInstallations {
		if installation.Password != "" {
			tmpDir := t.Task.GetTmpDir()
			vaultPasswordFile := path.Join(tmpDir, fmt.Sprintf("vault_%s_password", name))
			os.Remove(vaultPasswordFile) //nolint:errcheck
		}
	}
}
