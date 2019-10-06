# DVIRT19 assignment, in Rust

This assignment was completed as part of the Fall 2019 session of the Introduction to deployment and virtualization (DVIRT19) class at EPITA, Paris. The class was given by Joseph Chazalon and Clément Demoulins.

The assignment consists in wrapping an image histogram tool within a Docker image, and exposing it through a REST API. The size of the image counts towards 30% of the final grade.

I wasn't quite satisfied with the results of the assignment using Python, which made building a lean image unnecessarily difficult. Indeed, my first attempt clocked in at 1.05GB, and while my more committed classmates managed to shrink it down to around 150MB, I decided to try to reach for optimality. During the session, when asked how big the final image should be, Joseph jokingly told us "2 to 3MB". Which is an impossible goal in Python. In Rust, however...

Thus, I present to you my optimal solution. The resulting image is a mere 460KB in size, a 2400x decrease from my previous bloated solution.