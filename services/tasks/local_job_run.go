package tasks

import (
	"github.com/semaphoreui/semaphore/db_lib"
)

// Run запускает задачу
func (t *LocalJob) Run(username string, incomingVersion *string, alias string) (err error) {
	t.SetStatus("starting")
	t.Log("Starting job...")

	// Устанавливаем SSH ключи
	err = t.installSSHKeys()
	if err != nil {
		t.Log("Failed to install SSH keys: " + err.Error())
		return
	}
	defer t.clearSSHKeys()

	// Устанавливаем файлы Vault
	err = t.installVaultKeyFiles()
	if err != nil {
		t.Log("Failed to install Vault keys: " + err.Error())
		return
	}
	defer t.clearVaultKeyFiles()

	// Обновляем репозиторий
	err = t.updateRepository()
	if err != nil {
		t.Log("Failed to update repository: " + err.Error())
		return
	}

	// Переключаем на нужный коммит/ветку
	err = t.checkoutRepository()
	if err != nil {
		t.Log("Failed to checkout repository: " + err.Error())
		return
	}

	// Создаём приложение
	installingArgs := db_lib.LocalAppInstallingArgs{
		Username:        username,
		IncomingVersion: incomingVersion,
		Alias:           alias,
	}

	err = t.prepareRun(installingArgs)
	if err != nil {
		t.Log("Failed to prepare run: " + err.Error())
		return
	}

	// Запускаем приложение
	err = t.App.Run()
	if err != nil {
		t.Log("Failed to run app: " + err.Error())
		return
	}

	t.SetStatus("success")
	t.Log("Job completed successfully")

	return
}

// prepareRun подготавливает запуск задачи
func (t *LocalJob) prepareRun(installingArgs db_lib.LocalAppInstallingArgs) error {
	// Определяем тип приложения и создаём его
	switch t.Template.Type {
	case db.TemplateAnsible:
		t.App = db_lib.CreateAnsibleApp(t.Template, t.Inventory, t.Repository, t.Environment, t.Logger, t.KeyInstaller)
	case db.TemplateTerraform:
		t.App = db_lib.CreateTerraformApp(t.Template, t.Inventory, t.Repository, t.Environment, t.Logger, t.KeyInstaller)
	case db.TemplateShell:
		t.App = db_lib.CreateShellApp(t.Template, t.Inventory, t.Repository, t.Environment, t.Logger, t.KeyInstaller)
	default:
		t.App = db_lib.CreateLocalApp(t.Template, t.Inventory, t.Repository, t.Environment, t.Logger, t.KeyInstaller)
	}

	return t.App.Install(installingArgs)
}

// prepareRunTerraform подготавливает запуск Terraform
func (t *LocalJob) prepareRunTerraform(tfApp *db_lib.TerraformApp, installingArgs db_lib.LocalAppInstallingArgs, initArgs []string) error {
	return tfApp.InstallWithInit(installingArgs, initArgs)
}
