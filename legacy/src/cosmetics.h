#pragma once
#include "presentation.h"
namespace damage_numbers {
void cosmetics_init(void* context);
bool cosmetics_measure(const char*,float,VisualStyle,TextMetrics&);
void cosmetics_draw_effect(void* draw,const DrawNumber& number);
void cosmetics_draw(void* draw,const DrawNumber& number);
}
