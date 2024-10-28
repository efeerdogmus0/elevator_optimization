import elevator_lib as el


def run_simple_system():
    system = el.new_simple_system()

    while system.update():
        pass

    return (system.final_kwh(), system.get_total_wait_time())


def test_simple_system(count=10):
    total_kwh = 0
    total_wait_time = 0
    for _ in range(count):
        new_kwh, new_wait_time = run_simple_system()

        total_wait_time += new_wait_time
        total_kwh += new_kwh

    return total_kwh / count, total_wait_time / count



def main():
    avg_energy_consumption, avg_wait_time = test_simple_system(20)
    print("Simple system energy consumption average: ", avg_energy_consumption)
    print("Simple system wait time average: ", avg_wait_time)


if __name__ == "__main__":
    main()