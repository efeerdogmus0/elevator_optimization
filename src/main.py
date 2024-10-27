import elevator_lib as el

def main():
    system = el.new_simple_system()

    while True:
        system.update()
        elevators = system.get_elevators()

        print("python: energy usage: ", system.get_used_energy())
        print("python: elevator count: ", len(elevators))


if __name__ == "__main__":
    main()