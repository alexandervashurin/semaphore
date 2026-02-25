package bolt

import (
	"fmt"
)

type migration_2_14_7 struct {
	migration
}

// convertFloatToIntIfPossible конвертирует float в int, если возможно
// Аналог conv.ConvertFloatToIntIfPossible из pkg/conv
func convertFloatToIntIfPossible(v any) (int64, bool) {
	switch v := v.(type) {
	case float64:
		i := int64(v)
		if float64(i) == v {
			return i, true
		}
	case float32:
		i := int64(v)
		if float32(i) == v {
			return i, true
		}
	}
	return 0, false
}

func (d migration_2_14_7) Apply() (err error) {
	projectIDs, err := d.getProjectIDs()

	if err != nil {
		return
	}

	for _, projectID := range projectIDs {
		projectSchedules, err2 := d.getObjects(projectID, "schedule")
		if err2 != nil {
			return err2
		}

		for scheduleID, schedule := range projectSchedules {
			tplID, ok := convertFloatToIntIfPossible(schedule["template_id"])
			if !ok {
				return fmt.Errorf("schedule template id %s is not a valid integer", schedule["template_id"])
			}

			tpl, err3 := d.getObject(projectID, "template", string(intObjectID(int(tplID)).ToBytes()))
			if err3 != nil {
				return err3
			}

			if tpl == nil {
				err3 = d.deleteObject(projectID, "schedule", scheduleID)
			}

			if err3 != nil {
				return err3
			}
		}
	}

	return
}
