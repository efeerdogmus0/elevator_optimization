import elevator_lib as el


def run_simple_system():
    system = el.new_simple_system()

    while system.update():
        pass

    return system.final_kwh()


def test_simple_system(count=10):
    total_kwh = 0
    for _ in range(count):
        total_kwh += run_simple_system()
    return total_kwh / count



def main():
    print("Simple system energy consumption average: ", test_simple_system(10))


if __name__ == "__main__":
    main()