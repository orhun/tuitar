# Tuitar Hardware

> [!NOTE]
> Note that this document is for the v0 prototype.
> Please [report any issues](https://github.com/orhun/tuitar/issues) you find with it!

<!-- vim-markdown-toc GFM -->

- [Schematics](#schematics)
  - [How does this work?](#how-does-this-work)
- [PCB](#pcb)
- [Bill of Materials](#bill-of-materials)
  - [Capacitors](#capacitors)
  - [Resistors](#resistors)
  - [Semiconductors](#semiconductors)
  - [Connections](#connections)
  - [Mechanical](#mechanical)
- [Assembly](#assembly)
- [Post-Assembly](#post-assembly)
  - [Case](#case)
    - [Cutouts](#cutouts)
    - [Labels](#labels)

<!-- vim-markdown-toc -->

This directory contains the hardware design files for Tuitar.

- [PCB design](./pcb.json)
- [Schematic design](./schematics.json)

They are designed with the [EasyEDA](https://easyeda.com/) software, which you can use online or download the desktop client.

## Schematics

<img src="./assets/schematics.png" height="600">

➡️ Download the [**schematics JSON file here**](./schematics.json).

### How does this work?

The ESP-WROOM-32D module can capture the audio input from:

1. Microphone via the MAX4466 microphone amplifier.
2. Guitar via the mono audio jack.

The signal from the mono audio jack is amplified by the LM358P op-amp which is powered by 9V volts. The same 9V power is also reduced to 3.3V using the MP1584 buck converter to power the ESP-WROOM-32D and the rest of the components.

There is also a gain adjustment potentiometer (RP1) for the LM358P op-amp.

There are two arbitrary push buttons (SW1, SW2) for user interaction, a toggle switch (SW3) to turn the device on/off and an arbitrary potentiometer (RP2). An LED (LED1) is used to indicate the power status.

The ESP-WROOM-32D is connected to the 1.8" ST7735 TFT display where the UI is rendered.

## PCB

<img src="./assets/pcb-1.png" height="400">

<img src="./assets/pcb-2.png" height="400">

<img src="./assets/pcb-3.png" height="400">

➡️ Download the [**PCB JSON file here**](./pcb.json).

## Bill of Materials

### Capacitors

| Reference | Value | Qty |
| --------- | ----- | --- |
| **C1**    | 1 µF  | 1   |

### Resistors

| Reference      | Value                | Qty |
| -------------- | -------------------- | --- |
| **R1, R2, R5** | 1 kΩ                 | 3   |
| **R3, R4**     | 10 kΩ                | 2   |
| **RP1, RP2**   | 1 MΩ (potentiometer) | 2   |

### Semiconductors

| Reference | Value                 | Qty |
| --------- | --------------------- | --- |
| **D1**    | 1N5819 Schottky Diode | 1   |
| **LED1**  | Red LED               | 1   |
| **U1**    | LM358P Op-Amp         | 1   |
| **U3**    | MAX4466 Mic Amp       | 1   |
| **U4**    | MP1584 Buck Converter | 1   |
| **U6**    | ESP-WROOM-32D Devkit  | 1   |

### Connections

| Reference | Value                  | Qty |
| --------- | ---------------------- | --- |
| **J1**    | Mono Jack              | 1   |
| **9V**    | 9V Input Header        | 1   |
| —         | Female Pin Header 1×8  | 1   |
| —         | Female Pin Header 1×40 | 1   |
| —         | Male Pin Header 1×8    | 1   |

### Mechanical

| Reference    | Value                             | Qty |
| ------------ | --------------------------------- | --- |
| **SW3**      | On/Off Switch                     | 1   |
| **SW1, SW2** | Push Button                       | 2   |
| **LCD1**     | TFT 1.8" ST7735 Display (128×160) | 1   |
| —            | A kickass guitar                  | 1   |

## Assembly

> [!TIP]
> Some tips to make the assembly easier:
>
> - Lay out all components in groups
> - Solder the smallest/flattest components first (resistors, capacitors, diode)
> - Imagine this is lego and pls have fun!

1. Place the resistors (R1-R5), bend their legs slightly, solder and trim.

<img src="./assets/step-1.jpg" height="600">

2. Place the capacitor (C1) and do the same. Make sure the polarity is correct (the longer leg is positive).

<img src="./assets/step-2.jpg" height="600">

3. Place the diode (D1) and solder it. The stripe on the diode indicates the cathode (negative side).

> [!NOTE]
> The v0 prototype PCB has a mistake in the footprint which resulted in diode legs not fitting the holes.
> You need to solder it from the top side of the PCB, which is not ideal but works. Simply cut the legs shorter
> and align the diode with the PCB holes and solder it in place.

<img src="./assets/step-3.jpg" height="600">

4. Place the LED (LED1) and solder it. The longer leg is positive.

<img src="./assets/step-4.jpg" height="600">

5. Place the op-amp (U1) and solder it. Make sure the notch on the chip aligns with the notch on the PCB.

> [!TIP]
> Bend some legs slightly to hold the chip in place while soldering.

> [!NOTE]
> In the future it would be nicer to use a 8-pin socket for this instead of directly soldering on the PCB.

<img src="./assets/step-5.jpg" height="600">

6. Place the microphone amplifier (U3) and solder it. You might need to solder the 3-pin header pin first. Make sure the pins align with the PCB holes.

> [!WARNING]
> You might want to keep some space under the mic amp for reaching the gain adjustment potentiometer later.

<img src="./assets/step-6.jpg" height="600">

7. Solder header pins to the buck converter (U4), power it and adjust the output voltage to 3.3V, preferably on a breadboard. Then place it on the PCB and solder it in place.

> [!NOTE]
> You can solder it on one side and leave the other for measuring the voltage with multimeter. And then remove the pin header and solder it from both sides. It gets a bit difficult to solder with pin header in place. As always, make sure the labels on the PCB align with the pins on the buck converter.

<img src="./assets/step-7.jpg" height="600">

8. Place the female header pin for ESP-WROOM-32D (U6) and solder it.

<img src="./assets/step-8.jpg" height="600">

9. Solder wires to the mono jack (J1) and place it on the PCB.

> [!IMPORTANT]
> To find the correct pinout, you can use a multimeter as follows:
>
> - Set the multimeter to continuity mode.
> - Insert a guitar cable (or short wire) into the jack.
> - Touch one probe to the tip of the plug, then:
>   - Touch the other probe to each pin on the jack.
>   - The one that beeps is the Tip → this is the signal pin.
> - Touch the probe to the sleeve (outer metal part) of the plug:
>   - Touch the other probe to the remaining pins.
>   - The one that beeps is the Sleeve → this is the ground.

<img src="./assets/step-9.jpg" height="600">

10. Place the on/off switch (SW3) and solder it.

<img src="./assets/step-10.jpg" height="600">

11. Place the push buttons (SW1, SW2) and solder them.

<img src="./assets/step-11.jpg" height="600">

12. Place the potentiometers (RP1, RP2) and solder them. You can place them on the back side of the PCB to save space.

<img src="./assets/step-12.jpg" height="600">

13. Place the female pin header (1x8) for the display (LCD1) and solder it.

<img src="./assets/step-13.jpg" height="600">

> [!NOTE]
> The orientation of the display is quite inconvenient as of v0 prototype. It would be nicer if it was placed horizontally, but this is how it is for now. You can use a cable connector to extend the display if needed.

14. Solder the 9V input header (9V) to the PCB. Make sure the polarity is correct (the longer pin is positive).

<img src="./assets/step-14.jpg" height="600">

15. Celebrate with some cheese!

## Post-Assembly

Flash the firmware to the ESP-WROOM-32D using the USB connection.

➡️ See the [firmware documentation](../firmware/README.md) on how to set up the toolchain and flash the firmware.

Carefully place it on the board along with the TFT display. Make sure the display is aligned with the header pins.

<img src="./assets/step-15.jpg" height="600">

> [!CAUTION]
> The USB side of the ESP-WROOM-32D should be facing the edge of the PCB.

If everything goes well, you should see the Tuitar logo on the display and you can start rocking it.

<img src="./assets/step-16.jpg" height="600">

Enjoy!

<img src="./assets/step-17.jpg" height="600">

### Case

If you want to make a case for it, go for it!

The prototype case dimensions are **100mm x 65mm x 50mm** (L x W x H).

Some components such as buttons are extended via additional cable connectors, so you can place them on the front panel.

<img src="./assets/case-1.jpg" height="600">

<img src="./assets/case-2.jpg" height="600">

<img src="./assets/case-3.jpg" height="600">

<img src="./assets/case-4.jpg" height="600">

#### Cutouts

| Position / Reference | Purpose             | Qty | Notes                                                             |
| -------------------- | ------------------- | --- | ----------------------------------------------------------------- |
| Display              | TFT 1.8" ST7735     | 1   | Rectangular cutout on top, flush mount                            |
| Pots                 | 1 MΩ potentiometers | 2   | Front panel, centered under display, \~7–8 mm holes with knobs    |
| Buttons              | Push buttons        | 2   | Large red caps, placed symmetrically on each side of the panel    |
| On/Off Switch        | SK-12D02-VG5 toggle | 1   | Side-mounted, slot-shaped opening                                 |
| Microphone Input     | MAX4466 mic module  | 1   | Side-mounted, small round opening aligned with mic capsule        |
| Jack Input           | Mono audio jack     | 1   | Side-mounted, \~10–12 mm round hole                               |
| USB Connection       | ESP32 Devkit USB    | 1   | Rear or side rectangular cutout, accessible for programming/power |

#### Labels

To make the case look nice, you can print the labels below and stick them on the case.

The dimensions are **99mm x 64mm** (W x H).

<img src="./assets/case-labels.jpg" height="400">
