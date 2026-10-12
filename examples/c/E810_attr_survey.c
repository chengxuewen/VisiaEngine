/* E810 -- SDK consumption, C attribute-traversal survey -- the data-band acceptance example (zero new entries: the host panorama of the CAPI-04/10 family).
 *  If it does not run, the API is unfinished [E3D:D7].
 *
 *  park.geojson per entity: name(str) + fill-opacity(f64, the D9 original key) + a deliberate read of a missing key (missing != 0, contract in the act).
 *  ctest: example_E810_attr_survey (headless, the repo-root WORKING_DIRECTORY is guaranteed by the registration macro). */
#include <stdio.h>
#include <string.h>

#include "visiaengine.h"

int main(void) {
    uint64_t ve = visiaengine_create_headless(160, 120);
    if (!ve) { puts("E810 FAIL create"); return 1; }
    if (visiaengine_load_geojson(ve, "resources/data/park.geojson") != VE_OK) {
        printf("E810 FAIL load: %s\n", visiaengine_last_error(ve));
        return 1;
    }
    int n = visiaengine_entity_count(ve);
    int names = 0, opac = 0;
    for (int i = 0; i < n; ++i) {
        uint64_t ent = visiaengine_entity_at(ve, (uint32_t)i);
        char name[256];
        if (visiaengine_attr_str(ve, ent, "name", name, sizeof name) == 1) {
            ++names;
            double opacity = -1.0;
            int has_op = visiaengine_attr_f64(ve, ent, "fill-opacity", &opacity) == 1;
            if (has_op) ++opac;
            /* missing != zero contract: read a nonexistent key, out must hold its original value (here the sentinel -1.0) */
            double sentinel = -1.0;
            if (visiaengine_attr_f64(ve, ent, "definitely_missing_key", &sentinel) != 0 ||
                sentinel != -1.0) {
                puts("E810 FAIL the missing-key road polluted out");
                return 1;
            }
            printf("ATTR i=%d name=%s fill-opacity=%s\n", i, name,
                   has_op ? "" : "(none)");
        }
    }
    if (names < 2 || opac < 1) {
        printf("E810 FAIL attribute face shrunken names=%d opac=%d / n=%d\n", names, opac, n);
        return 1;
    }
    if (visiaengine_destroy(ve) != VE_OK) { puts("E810 FAIL destroy"); return 1; }
    printf("OK E810_attr_survey names=%d opacity=%d entities=%d\n", names, opac, n);
    return 0;
}
