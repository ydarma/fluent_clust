# Fluent Data

A low footprint streaming data modelization library and service.

The algorithm reads data points from an input stream, fits the model and sends the updated model to an output stream.

The online algorithm fits a model that consists in a set of balls. Each of each is described by its center, radius and weight
(the decayed number of points that were included in the ball).

# Install and run the program
Install :
```
cargo install fluent_data
```

Run the program and enter data points in the standard input. The program will answer with a model:
```
fluent_data
[5,-1]
[{"center":[5.0,-1.0],"radius":null,"weight":0.0}]
[1,1]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":1.0}]
[15,-13]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.95},{"center":[18.5,-16.5],"radius":3.9597979746446663,"weight":1.0}]
[11,23]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.9025},{"center":[18.5,-16.5],"radius":3.9597979746446663,"weight":0.95},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":1.0}]
[31,-3]    
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.8573749999999999},{"center":[18.5,-16.5],"radius":3.9597979746446663,"weight":0.9025},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":0.95},{"center":[34.125,0.375],"radius":3.6796738985948196,"weight":1.0}]
[10,-9]    
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.8145062499999999},{"center":[14.032194480946124,-12.557818659658345],"radius":8.65915237435586,"weight":1.9024999999999999},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":0.9025},{"center":[34.125,0.375],"radius":3.6796738985948196,"weight":0.95}]
[6,-4]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.7737809374999999},{"center":[11.264857881136951,-9.609388458225668],"radius":9.828917387831996,"weight":2.9025},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":0.8573749999999999},{"center":[34.125,0.375],"radius":3.6796738985948196,"weight":0.9025}]
[-2,-5]
[{"center":[6.7297134962820016,-6.8681649994430005],"radius":15.539441192890935,"weight":4.6762809375},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":0.8145062499999999},{"center":[34.125,0.375],"radius":3.6796738985948196,"weight":0.8573749999999999}]
```

A model is represented as a json array with an object for each ball:
 - `center` is the center of the ball,
 - `radius` is the radius of the ball,
 - `weight` is the weight of the ball (the probability is obtained by dividing the weight by the sum of weights).
 
## Running as a service
The program can be run as a websocket server:
```
fluent_data --service
```
Data points are sent to `ws://0.0.0.0:9001/ws/points` and model are received from `ws://0.0.0.0:9001/ws/models`.
The port can be customized by setting the `PORT` environment variable.

For sending and receiving points, the websocket client [websocat](https://crates.io/crates/websocat) can be used.
Open a first terminal that will listen for models:
```
websocat ws://127.0.0.1:9001/ws/models
```
Open a second terminal and enter some points:
```
websocat ws://127.0.0.1:9001/ws/points
[5,-1]
[1,1]
[15,-13]
[11,23]
[31,-3]    
[10,-9]    
[6,-4]
[-2,-5]
```
The first terminal should display models:
```
[{"center":[5.0,-1.0],"radius":null,"weight":0.0}]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":1.0}]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.95},{"center":[18.5,-16.5],"radius":3.9597979746446663,"weight":1.0}]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.9025},{"center":[18.5,-16.5],"radius":3.9597979746446663,"weight":0.95},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":1.0}]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.8573749999999999},{"center":[18.5,-16.5],"radius":3.9597979746446663,"weight":0.9025},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":0.95},{"center":[34.125,0.375],"radius":3.6796738985948196,"weight":1.0}]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.8145062499999999},{"center":[14.032194480946124,-12.557818659658345],"radius":8.65915237435586,"weight":1.9024999999999999},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":0.9025},{"center":[34.125,0.375],"radius":3.6796738985948196,"weight":0.95}]
[{"center":[1.0,1.0],"radius":4.47213595499958,"weight":0.7737809374999999},{"center":[11.264857881136951,-9.609388458225668],"radius":9.828917387831996,"weight":2.9025},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":0.8573749999999999},{"center":[34.125,0.375],"radius":3.6796738985948196,"weight":0.9025}]
[{"center":[6.7297134962820016,-6.8681649994430005],"radius":15.539441192890935,"weight":4.6762809375},{"center":[13.5,28.5],"radius":4.833218389437829,"weight":0.8145062499999999},{"center":[34.125,0.375],"radius":3.6796738985948196,"weight":0.8573749999999999}]
```
 
# Using the library

See [the crate documentation](https://docs.rs/fluent_data/latest/fluent_data/).

# Customizing the algorithm

See [the customization section of the crate documentation](https://docs.rs/fluent_data/latest/fluent_data/index.html#customization).

# How it works
Given a set of balls fitted from data points received so far, let `P` be the new incoming point.
Let `B` be the ball that most probably contains `P` (*).

Let `C` be the center of `B`, `r` its radius and `w` its weight. Let `d` be the distance from `P` to `C`: `d = |PC|`.

(*) by "most probably includes" we mean that `B` minimizes the quantity `d/r` for all balls in the model.

The fitting algorithm is the following:
 - (I) If the distance is less than four times `B` radius, `d < 4r`,
   - the new point belongs to `B`, `B` is incrementally updated:
      - the sqaure of the radius is set to the average square radius: `r² <- (w.r² + d²) / (w + 1)`,
      - the center is set to the average center: `C <- (w.C + P) / (w + 1)`,
      - the weight is incremented by 1: `w <- w + 1`.
   - otherwise a new ball `B*` is created:
      - the radius is set to 1/5 of the distance: `r* <- d / 5`, i.e. `r*² <- d² / 25`,
      - the center `C*` is set such as `C`, `P` and `C*` are aligned and `CC* = 6/5 CP`, i.e. `C* <- (-C + 5P) / 6`
      - the weight of the ball is set to 1.
 - (II) Let `B'` be the nearest ball of `B`, i.e. the distance `CC'` is minimal among all balls in the model.
        In the first case above, `B` and `B'` may be merged into a single ball
        if the square distance between them is less than the sum of square of their radius, `d² < r² + r'²`:
   - `B` is updated:
      - the square of the radius is set to the weighted average of the square of the radius, plus the square of the distance `r² <- (w.r² + w'.r'²) / (w + w') + d²`,
      - the center is set to the weighted average of the centers `C <- (w.C + w'.C') / (w + w')`,
      - the weight is set to the sum of the weights `w <- w + w'`.
   - `B'` is dropped.
 - (III) The weight of all alls but the one which `P` belongs to (that is `B` or `B*`) are decayed with a factor of 0.95, `w <- 0.95 w`.
   - All balls which weight is lower than 1/100, `w < 1/100` are removed.

## About the implementation
Each ball `B` is represented by `(C, r, w)` respectively the center,
square of the radius (which is more useful for computations) and weight of `B`.

The model is represented in memory with a graph which vertices are associated with balls.
For a vertex `V` associated with ball `B`, `V` neighborhood is `{V', V"}`, the vertices associated with the 2 nearest balls of `B`, 
i.e. the two balls `{B', B"}` for which distances `|CC'|` a `|CC"|` are respectively
the smallest and second smallest among all balls in the model.

Given a set of balls fitted from data points received so far, let `P` be the new incoming point.
Let `{B, B°}` be the two balls that most probably contain `P` (* see above) and `{V, V°}` the coresponding vertices.

The graph is maintained in memory as follows:
 - When a new ball `B*` is created at step (I), a new vertex `V*` is created with associated ball `B*`, `{V, V°}` become `V*` neighborhood.
   `B*` may be now closer from `B` than `B'` or `B"`.  
   Thus, `V` neighbors are recomputed among `{V', V", V*}`, by searching the 2 nearest neghbors of `B` among `{B', B", B*}`.
 - When `B` is updated to include `P` at step (I), `B°` may be now closer from `B` than `B'` or `B"`:
    - if `B` and `B'` are not merged at step (II) and `B°` does not belong to `{B', B"}`, `V` neighbors are recomputed among `{V', V", V°}`, by searching the 2 nearest neghbors of `B` among `{B', B", B°}`.  
    - if `B` and `B'` are merged at step (II): if `B°` does not belong to `{B', B"}` the neighborhood of `V` becomes `{V", V°}` otherwise it becomes `{V"}`.

The graph implementation is private to the crate, its implementation can be found here: [graph.rs](https://github.com/ydarma/fluent_data/blob/main/src/graph.rs).