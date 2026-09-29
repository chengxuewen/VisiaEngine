#!/usr/bin/env bash
# window-probe-all.sh — 全部常驻窗例的第一帧守卫批量（黑屏三连根治的 CI 化）。
# 每例：零参跑 → Xvfb 截屏 → 亮度比断言（window-probe.sh 单例形）。
# 自退例不在此列（golden 像素门守它们）。一例红=整链红（报文指名）。
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
RC=0
for E in E101_clear E201_load_gltf E202_geo_viewer E205_text_labels \
         E301_switch_camera E302_fly_camera E303_split_screen \
         E304_projection_morph E306_map_controls E402_pick_interactive \
         E403_box_select E501_shadow_demo E502_color_tuning \
         E503_stroke_points E504_glass_water E505_cjk_labels \
         E506_postprocessing E507_hdr_tonemap E508_pbr_materials E509_ssao; do
  bash scripts/window-probe.sh "$E" || RC=1
done
[ $RC -eq 0 ] && echo "WINDOW-PROBE-ALL ✓ (20 resident windows)" || echo "WINDOW-PROBE-ALL ✗"
exit $RC
