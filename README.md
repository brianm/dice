# dice

Rolls dice using a small expression language:

The simplest expression is just a number, indicating to roll
a die with that many sides, ie: `dice 20` or `dice d20` to roll a 20 sided die.

If you want to roll multiple dice you can specify how many with a prefix,
for example three dice with six sides each would be `3d6`.

You can then specify how many dice to keep or drop from the roll. To drop dice
use `d` or `D` to drop low rolls or high rolls respectively. For example,
`4d6d1` says to "roll four dice with six sides dropping the lowest die", whereas
`2d20D1` says to "roll two dice with twenty sides each dropping the higher one".

The same thing works for keep with `k` and `K`, with `k` meaning to keep higher
rolls and `K` to keep lower rolls. This is different from `d` and `D`. Basically
it defaults (lower case) to the belief you want high rolls, therefore that is
easier to type (no need for the `shift` to get capital) :-) If you find it annoying
to use, let me know and I'll consider changing it in a future version.

Finally, you may add a constant modifier to the roll by appending `+` or `-` and
a value, such as `4d6+1` `3d6-2` or `2d20K1+7`

You can also send multiple expressions:

`dice 4d6d1 4d6d1 4d6d1 4d6d1 4d6d1 4d6d1`

In summary:

```
    d20     1 x d20
    3d6     3 x d6
    4d6d1   3 x d6 dropping lowest
    20+1    1 x d20 and add one to the result   
    2d8K1-1 2 x d8 keep the higher and subtract 1
```

Trivial die rolling expression parser, exists primarily to play with
[pest](https://pest.rs/) and because my son wanted one :-)
