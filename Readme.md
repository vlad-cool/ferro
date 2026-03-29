# Ferro
Ferro is verilog-like hardware description language idea that brings modern rust-like syntax, flexible configuation and embedding constraints

Goals:

- [ ] Syntax
- [ ] Verilog translator
- [ ] Full synthesis tool
- [ ] Simulation tool
- [ ] Interactive simulatiom tool (GTK-Wave like, where input signals can be changed on the go)

Implemented: 0%

# Syntax

## Code example

```
module top (
    clk: clock<pin: A1, ..., frequency: 27MHz>,
    rst: reset<pin: B2, ..., active: low>,

    leds: output<width: 6, pins: [A3, B4, A5, B6, A7, B8], active: low>,
    dbg_led: output<pin: C2, active: low>,
    btn: input<pin: C3>,
)
{
    led_blink<leds_count: 6> LedBlink (
        clk: clk,
        rst: rst,
        leds: leds,
        next_mode: btn,
    );

    dbg_led = led_blink.sec_counter[0];
}

module<leds_count: uint> led_blink (
    clk: clock, // Seperate names for input clocks and resets
    rst: reset,
    leds: output<width: data_width>, // Shorthand for output[data_width-1:0] data_in
    next_mode: input, // Trailing commas allowed
)
{
    #module_cfg(sync_reset);
    
    #[encoding(gray)]
    enum Modes {
        Counter,
        Running,
        #[main_value] // Forbidden values are binded to this value, by default first one
        Chess,
    }

    let mode: Modes;

    let counter: logic<max: clk.frequency>; // Clocks can store frequency
    let sec_counter: logic<width: leds_count>; // SV alterenative: logic[leds_count-1:0];
    let shift_reg: logic<width: leds_count>;

    #[async_reset]
    ff { // Shorthand for ff (clock: clk, reset: rst), Clock and reset may be ommited if it is one in the module
        reset {
            mode = Mode::Counter; // All assignments are non-blocking
        }

        if next_mode { // Parentheses may be ommited in ifs
            mode = match mode { // Rust-like match case
                Counter => Running, // Enum name may be ommited? (to be discussed)
                Running => Chess,
                Chess => Counter,
            };
        }
    }

    ff {
        reset {
            counter = 0;
        }
        counter += 1; // Can't exceed max value; verilog alternative: counter <= counter == FREQUENCY - 1 ? 0 : counter + 1
    }

    ff {
        reset {
            sec_counter = 0
        }
        if counter == 0 {
            sec_counter += 1;
        }
    }

    ff {
        reset {
            shift_reg = 1;
        }
        if counter == 0 {
            shift_reg = [shift_reg[0], shift_reg >> 1]; // Concatenation
        }
    }

    leds = match mode {
        Counter => sec_counter,
        Running => shift_reg,
        Chess => {
            let pattern: logic<width: leds_count> = if (leds_count % 2 == 0) { // Local variables allowed, static ifs
                [[2'b10] * (leds_count / 2)]
            }
            else {
                [1'b1, [2'b10] * (leds_count / 2)]
            }

            if sec_counter[0] {
                pattern
            }
            else {
                ~pattern
            }
        },
    }
}

module static_pll<div: uint, mul: uint> (
    clk_in: clock,
    clk_init: clock,
    clk_out: output_clock<frequency: clk_in.frequency / div * mul>,
)
{
    static_assert(clk_in >= 1, clk_in <= 400);
    static_assert(clk_init == 50);
    ...
}

module dynamic_pll<div: uint, mul: uint> (
    clk_in: clock<frequency in [1MHz..400MHz]>
    clk_out: output_clock
)
{
    ...
}

```

## Bus operations:

```
let b: logic<width: 10> = 10'h2AA;
let a: logic = xor(logic);
```

logic can have modifiers (in <>):

- width
- max_value (static value)
- min_value (static value)
- multiple_assignment: (X (default), or, and, xor), supported for module instance output
- static: compile-time constant, throw compilation error if value can't be calculated in compile-time
