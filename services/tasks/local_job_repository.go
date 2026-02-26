package tasks

import (
	"github.com/semaphoreui/semaphore/db_lib"
)

// updateRepository обновляет репозиторий
func (t *LocalJob) updateRepository() error {
	gitClient := db_lib.CreateGitClient(t.Repository, t.Logger, t.KeyInstaller)
	return gitClient.CloneOrPull()
}

// checkoutRepository переключает репозиторий на нужный коммит/ветку
func (t *LocalJob) checkoutRepository() error {
	gitClient := db_lib.CreateGitClient(t.Repository, t.Logger, t.KeyInstaller)

	if t.Task.CommitHash != nil {
		return gitClient.Checkout(*t.Task.CommitHash)
	}

	return nil
}
