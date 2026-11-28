#pragma once
#include <stdint.h>
typedef uint8_t UBYTE;
typedef uint16_t UWORD;
typedef uint32_t UDOUBLE;
enum { EPD_RST_PIN=0, EPD_DC_PIN=1, EPD_CS_PIN=2, EPD_BUSY_PIN=3 };
void DEV_Digital_Write(UWORD, UBYTE);
UBYTE DEV_Digital_Read(UWORD);
void DEV_SPI_WriteByte(UBYTE);
void DEV_Delay_ms(UDOUBLE);
