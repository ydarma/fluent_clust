# Fluent Data

A low footprint streaming data modelization library and service.

The algorithm reads data points from an input stream, fits the model and sends the updated model to an output stream.

The online algorithm fits a mixed Gaussian model. Components covariances are supposed to be zero, i.e. for a given component dimensions are independant from each other. Theese are very strong hypothesis, thus the algorithm is not suited to all kind of data.

# Install and run the program
Install :
```
cargo install fluent_data
```

Run the program and enter data points in the standard input. The program will answer with a model:
```
fluent_data
[5,-1]
[{"mu":[5.0,-1.0],"sigma":null,"weight":0.0}]
[1,1]
[{"mu":[1.0,1.0],"sigma":20.0,"weight":1.0}]
[15,-13]
[{"mu":[1.0,1.0],"sigma":20.0,"weight":0.95},{"mu":[18.5,-16.5],"sigma":15.68,"weight":1.0}]
[11,23]
[{"mu":[1.0,1.0],"sigma":20.0,"weight":0.9025},{"mu":[18.5,-16.5],"sigma":15.68,"weight":0.95},{"mu":[13.5,28.5],"sigma":23.36,"weight":1.0}]
[31,-3]    
[{"mu":[1.0,1.0],"sigma":20.0,"weight":0.857375},{"mu":[18.5,-16.5],"sigma":15.68,"weight":0.9025},{"mu":[13.5,28.5],"sigma":23.36,"weight":0.95},{"mu":[34.125,0.375],"sigma":13.54,"weight":1.0}]
[10,-9]    
[{"mu":[1.0,1.0],"sigma":20.0,"weight":0.81450625},{"mu":[14.032194480946124,-12.557818659658345],"sigma":74.98091984231274,"weight":1.9025},{"mu":[13.5,28.5],"sigma":23.36,"weight":0.9025},{"mu":[34.125,0.375],"sigma":13.54,"weight":0.95}]
[6,-4]
[{"mu":[1.0,1.0],"sigma":20.0,"weight":0.7737809375},{"mu":[11.264857881136951,-9.609388458225668],"sigma":96.60761701682615,"weight":2.9025},{"mu":[13.5,28.5],"sigma":23.36,"weight":0.857375},{"mu":[34.125,0.375],"sigma":13.54,"weight":0.9025}]
[-2,-5]
[{"mu":[6.7297134962820016,-6.8681649994430005],"sigma":241.4742325873156,"weight":4.6762809375},{"mu":[13.5,28.5],"sigma":23.36,"weight":0.81450625},{"mu":[34.125,0.375],"sigma":13.54,"weight":0.857375}]
```

A model is represented as a json array with an object for each component:
 - `mu` is the mean of the component,
 - `sigma` is the variance of the component,
 - `weight` is the weight of the component (the probability is obtained by deviding the weight by the sum of weights).
 
The program can be run as a websocket server:
```
fluent_data --service
```
Data points must be sent to `ws://0.0.0.0:9001/ws/points` and model are received from `ws://0.0.0.0:9001/ws/models`.
The port can be customized by setting the `PORT` environment variable.
 
# Using the library

See [the crate documentation](https://docs.rs/fluent_data/latest/fluent_data/).

# Customizing the algorithm

See [the customization section of the crate documentation](https://docs.rs/fluent_data/latest/fluent_data/index.html#customization)

# How it works
Given a mixed model fitted from data points received so far, whe the new point `P` arrives:
 - The two most probable component of the mixed model are retrieved.
 - If the distance to the most probable mean `C` is less than 4 times its standard deviation,
   - the new point belongs to this component: the component standard deviation and mean are updated incrementally,
     its weight is increased by 1;
   - otherwise a new component is created:
     - the standard deviation is set to 1/5 of the distance to the closest component,
     - the mean `M` is set to a distance of the closest mean equal to 6 times the new standard deviation (`CP = 6/5 CM`),
     - the weight of the component is set to 1.
 - The weight of all components but the one which the ne point belongs to are decreased by a factor of 0.95.
 - All components which weight are lower than 1/100 are removed.