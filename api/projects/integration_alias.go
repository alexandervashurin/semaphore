package projects

import (
	"crypto/rand"
	"encoding/hex"
	"github.com/semaphoreui/semaphore/api/helpers"
	"github.com/semaphoreui/semaphore/db"
	"github.com/semaphoreui/semaphore/util"
	"net/http"
)

type publicAlias struct {
	ID  int    `json:"id"`
	URL string `json:"url"`
}

func getPublicAlias(alias db.IntegrationAlias) publicAlias {

	return publicAlias{
		ID:  alias.ID,
		URL: util.GetPublicAliasURL("integrations", alias.Alias),
	}
}

func getPublicAliases(aliases []db.IntegrationAlias) (res []publicAlias) {

	res = make([]publicAlias, 0)
	for _, alias := range aliases {
		res = append(res, getPublicAlias(alias))
	}

	return
}

func GetIntegrationAlias(w http.ResponseWriter, r *http.Request) {
	project := helpers.GetFromContext(r, "project").(db.Project)
	integration, ok := helpers.GetFromContext(r, "integration").(db.Integration)

	var integrationId *int
	if ok {
		integrationId = &integration.ID
	}

	aliases, err := helpers.Store(r).GetIntegrationAliases(project.ID, integrationId)

	if err != nil {
		helpers.WriteError(w, err)
		return
	}

	helpers.WriteJSON(w, http.StatusOK, getPublicAliases(aliases))
}

func AddIntegrationAlias(w http.ResponseWriter, r *http.Request) {
	project := helpers.GetFromContext(r, "project").(db.Project)
	integration, ok := helpers.GetFromContext(r, "integration").(db.Integration)

	var integrationId *int
	if ok {
		integrationId = &integration.ID
	}

	// Генерация случайного алиаса
	randomBytes := make([]byte, 16)
	rand.Read(randomBytes)
	aliasValue := hex.EncodeToString(randomBytes)[:16]

	alias, err := helpers.Store(r).CreateIntegrationAlias(db.IntegrationAlias{
		Alias:         aliasValue,
		ProjectID:     project.ID,
		IntegrationID: integrationId,
	})

	if err != nil {
		helpers.WriteError(w, err)
		return
	}

	helpers.WriteJSON(w, http.StatusOK, getPublicAlias(alias))
}

func RemoveIntegrationAlias(w http.ResponseWriter, r *http.Request) {
	project := helpers.GetFromContext(r, "project").(db.Project)
	aliasID, err := helpers.GetIntParam("alias_id", w, r)

	if err != nil {
		helpers.WriteError(w, err)
		return
	}

	err = helpers.Store(r).DeleteIntegrationAlias(project.ID, aliasID)

	if err != nil {
		helpers.WriteError(w, err)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}
