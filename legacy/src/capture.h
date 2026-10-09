// Capture lifecycle shared by the Forge entry points. Panel visibility controls
// settings only; the free-render callback continues consuming and drawing hits.

#pragma once
#include <cstdint>

namespace damage_numbers {
void capture_install();
void capture_monitor();
void capture_panel();
void capture_consume();
void capture_render();
void capture_reset_placement();
} // namespace damage_numbers
