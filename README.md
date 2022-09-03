# Fluent Data

A low footprint streaming data modelization library and service.

The algorithm reads data points from an input stream, fits the model and sends the updated model to an output stream.

The online algorithm fits a mixed Gaussian model. Components covariances are supposed to be zero, i.e. for a given component dimensions are independant from each other. Theese are very strong hypothesis, thus the algorithm is not suited to all kind of data.

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