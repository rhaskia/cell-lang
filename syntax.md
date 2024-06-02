# Memory Defining
~: starts memory
identifier/expr: represents what the memory is
; : optional, used for repeating. is followed by a number
~ ends memory

# Functions
Functions can be defined with |>.
They are used to change data (ints/bools) (although these are interchangeable)
examples:
|> f(a) = #a>=3 || r(a)
|> r(a) = sum(@n, @w) > 3 >= 3
|> o(a) = 3*a 

# Main Coroutine
main defines the conditionals and outcomes
It goes as such:
centre_value: optional_conditional [|> or $] outcome
centre_value can be replaced with a number to pattern match the centre value, similar to haskell
otherwise conditionals can be done through the conditional part of the coroutine
the triangle or pipeline operator here indicates a change in value, while the sign indicates a printing of a value
@example: 3 * 8 == @north |> @left * 30

# Directionals
@ can be used to "load" a directional, or cell in a certain direction from the centre.
values include:
@n(orth), @s(outh), @e(ast), @w(est)
@ne, @se, @nw, @sw
@all
@diag(onals)
@dir(ect)
Some of these return array values, and must be summed using ```=[value]```
Those of these that are arrays can accessed randomly by a ?

# Counts
`#[conditional]` returns the count of a conditional, eg:
`#[@all==1]` returns the amount of cells that equal one
`#[@dir>1]` returns the amount of direct cells that are greater than one

# Sums
```=[value]``` sums the values of a array or conditional, eg:
```=[@all]``` returns the sum of all cells around it 
