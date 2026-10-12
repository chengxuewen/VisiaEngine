# CARLA / esmini — the direct reference for ODR/OSC visualization

**Snapshot 2026-09-03, measured**: CARLA 14k★ (active yesterday, MIT); **esmini/esmini 942★** (note the org spelling).

## Profile
- **CARLA**: a UE4-based AV simulator — an OpenDRIVE **consumer**, proof that the demand is large; its shape (game engine + heavy sim) is the opposite of Visia.
- **esmini**: an independent C++ tool in ASAM's official niche (OpenDRIVE/OpenSCENARIO reading, road mesh generation, scenario playback, OSI interface; **its rendering layer is OSG** — OSG again). **Small, focused, readable through** (BSD-2 *recheck*).

## For VisiaEngine (Beta-phase ODR/OSC plugin = this file's only purpose)
- **Adopt (high)**: ① esmini's **road->mesh generation logic** (reference line + lane width/superelevation/cross & longitudinal profile -> triangle strips, junction connections) is exactly the algorithmic requirement spec for Visia's "auto-generate roads", BSD allows internalized reference; ② the layered parsing of OpenDRIVE XML (progressive loading: geometry segments first, then lanes, then signals); ③ the scenario player (OSC events -> entity state timelines) = the twin's temporal-dynamic model (benchmarked against cesium's SampledProperty) in the AV domain; ④ CARLA's existence proves: when customers use Visia as a "simulation visualization base", their mental reference is CARLA's look.
- **Avoid**: ① esmini's OSG rendering (old) and API shape (C++ singletons everywhere) — read only the algorithm layer; ② the spec-completeness trap (ODR 300+ pages; first support the <20% core elements: planView/elevation/laneSection/link/signal, the rest go into enterprise-plugin paywall candidates); ③ stay out of sensor simulation (the inflation path of sim promises on the whitepaper).
- **Current-state ruling**: zero relevance for MVP (the whitepaper already demoted ODR/OSC to a Beta plugin — see D3 revision ②); when Beta starts this file is promoted to design input, at which point an esmini source walkthrough + ODR-parsing crate survey (the `road` pure-Rust crates' current state unverified).

## Sources
shields (2026-09-03); github.com/esmini/esmini and CARLA docs are public.
