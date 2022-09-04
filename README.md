# Fluent Data

A low footprint streaming data modelization library and service.

The algorithm reads data points from an input stream, fits the model and sends the updated model to an output stream.

The online algorithm fits a model that consist in a set of balls. Each of each is described by its center, radius and weight.

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
[{"center":[1.0,1.0],"radius":20.0,"weight":1.0}]
[15,-13]
[{"center":[1.0,1.0],"radius":20.0,"weight":0.95},{"center":[18.5,-16.5],"radius":15.68,"weight":1.0}]
[11,23]
[{"center":[1.0,1.0],"radius":20.0,"weight":0.9025},{"center":[18.5,-16.5],"radius":15.68,"weight":0.95},{"center":[13.5,28.5],"radius":23.36,"weight":1.0}]
[31,-3]    
[{"center":[1.0,1.0],"radius":20.0,"weight":0.857375},{"center":[18.5,-16.5],"radius":15.68,"weight":0.9025},{"center":[13.5,28.5],"radius":23.36,"weight":0.95},{"center":[34.125,0.375],"radius":13.54,"weight":1.0}]
[10,-9]    
[{"center":[1.0,1.0],"radius":20.0,"weight":0.81450625},{"center":[14.032194480946124,-12.557818659658345],"radius":74.98091984231274,"weight":1.9025},{"center":[13.5,28.5],"radius":23.36,"weight":0.9025},{"center":[34.125,0.375],"radius":13.54,"weight":0.95}]
[6,-4]
[{"center":[1.0,1.0],"radius":20.0,"weight":0.7737809375},{"center":[11.264857881136951,-9.609388458225668],"radius":96.60761701682615,"weight":2.9025},{"center":[13.5,28.5],"radius":23.36,"weight":0.857375},{"center":[34.125,0.375],"radius":13.54,"weight":0.9025}]
[-2,-5]
[{"center":[6.7297134962820016,-6.8681649994430005],"radius":241.4742325873156,"weight":4.6762809375},{"center":[13.5,28.5],"radius":23.36,"weight":0.81450625},{"center":[34.125,0.375],"radius":13.54,"weight":0.857375}]
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
[{"center":[1.0,1.0],"radius":20.0,"weight":1.0}]
[{"center":[1.0,1.0],"radius":20.0,"weight":0.95},{"center":[18.5,-16.5],"radius":15.68,"weight":1.0}]
[{"center":[1.0,1.0],"radius":20.0,"weight":0.9025},{"center":[18.5,-16.5],"radius":15.68,"weight":0.95},{"center":[13.5,28.5],"radius":23.36,"weight":1.0}]
[{"center":[1.0,1.0],"radius":20.0,"weight":0.857375},{"center":[18.5,-16.5],"radius":15.68,"weight":0.9025},{"center":[13.5,28.5],"radius":23.36,"weight":0.95},{"center":[34.125,0.375],"radius":13.54,"weight":1.0}]
[{"center":[1.0,1.0],"radius":20.0,"weight":0.81450625},{"center":[14.032194480946124,-12.557818659658345],"radius":74.98091984231274,"weight":1.9025},{"center":[13.5,28.5],"radius":23.36,"weight":0.9025},{"center":[34.125,0.375],"radius":13.54,"weight":0.95}]
[{"center":[1.0,1.0],"radius":20.0,"weight":0.7737809375},{"center":[11.264857881136951,-9.609388458225668],"radius":96.60761701682615,"weight":2.9025},{"center":[13.5,28.5],"radius":23.36,"weight":0.857375},{"center":[34.125,0.375],"radius":13.54,"weight":0.9025}]
[{"center":[6.7297134962820016,-6.8681649994430005],"radius":241.4742325873156,"weight":4.6762809375},{"center":[13.5,28.5],"radius":23.36,"weight":0.81450625},{"center":[34.125,0.375],"radius":13.54,"weight":0.857375}]
```
 
# Using the library

See [the crate documentation](https://docs.rs/fluent_data/latest/fluent_data/).

# Customizing the algorithm

See [the customization section of the crate documentation](https://docs.rs/fluent_data/latest/fluent_data/index.html#customization).

# How it works
Given a set of balls fitted from data points received so far, whe the new point `P` arrives:
 - (I) If the distance to the center `C` of the ball `B` that most probably include `P` is less than 4 times the radius of `B`,
   - the new point belongs to `B`: `B` centers and radius are updated by incremental average,
     its weight is incremented by 1;
   - otherwise a new ball `B'` is created:
     - the radius is set to 1/5 of the distance from `P` to `C` (`1/5 CP`),
     - the center `C'` is set to a distance from `C` equal to 6 times the radius of `B'` (`CC' = 6/5 CP`),
     - the weight of the ball is set to 1.
 - (II) In the first case above, the two balls `B` and `B2` of centers `C` and `C2` that most probably include `P` are merged into a single ball if their distance is
   less than the sum of their radius:
     - the radius is the weighted average of the radius of `B` and `B2`, plus the distance `CC2`,
     - the center is the weighted average of the centers of `C` and `C2`,
     - the weight is the sum of the weights.
 - (III) The weight of all alls but the one which `P` belongs to are decayed with a factor of 0.95.
   - All balls which weight are lower than 1/100 are removed.
 
 ## About the implementation
 The model is represented in memory with a graph which vertices are model balls.
 Edges links the 2 nearest neighbors of each balls (i.e. the two balls that could be most probably the same).
 
 The graph is maintained in memory:
   - When a new ball is created, its two most probable balls become the new vertice neighbors.
   - When the incoming point is merged the ball neighborhood is recomputed using
     the second most probable ball of the incoming point which can be interspersed in the current neighborhood.