#!/usr/bin/env bash
# window-probe-all.sh -- the batch first-frame guard over all resident-window examples (the CI-ized form of the black-screen trio cure).
# Per example: run with no args -> Xvfb screenshot -> lit-ratio assert (the window-probe.sh single-example shape).
# Self-exiting examples are not in this list (their golden-pixel gates guard them). One example red = the whole chain red (the message names it).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
RC=0
for E in E101_clear E201_load_gltf E202_geo_viewer E205_text_labels \
         E301_switch_camera E302_fly_camera E303_split_screen \
         E304_projection_morph E306_map_controls E402_pick_interactive \
         E403_box_select E501_shadow_demo E502_color_tuning \
         E503_stroke_points E504_glass_water E505_cjk_labels E510_route_flow E511_gate \
         E506_postprocessing E507_hdr_tonemap E508_pbr_materials E509_ssao; do
  bash scripts/window-probe.sh "$E" || RC=1
done
[ $RC -eq 0 ] && echo "WINDOW-PROBE-ALL ✓ (20 resident windows)" || echo "WINDOW-PROBE-ALL ✗"
exit $RC
