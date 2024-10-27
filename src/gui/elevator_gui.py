# elevator_gui.py
import socket
import json
import sys
import customtkinter as ctk
from tkinter import Menu, BooleanVar, ttk, Canvas, messagebox
import threading
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
from datetime import datetime

class SimulationClient:
    def __init__(self):
        self.socket = None
        self.connect_to_server()

    def connect_to_server(self):
        """Establish a socket connection to the Rust server."""
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            self.socket.connect(('127.0.0.1', 9000))
            self.socket.settimeout(5)  # Set a timeout for socket operations
        except socket.error as e:
            messagebox.showerror("Connection Error", f"Failed to connect to Rust server: {e}")
            sys.exit(1)

    def send_request(self, request):
        """Send a JSON request to the Rust server and receive the response."""
        try:
            request_json = json.dumps(request)
            self.socket.sendall(request_json.encode('utf-8'))
            # Receive response
            response_data = self.socket.recv(16384)  # Adjust buffer size as needed
            response_json = response_data.decode('utf-8')
            response = json.loads(response_json)
            return response
        except socket.timeout:
            messagebox.showerror("Timeout Error", "Request to Rust server timed out.")
            return None
        except socket.error as e:
            messagebox.showerror("Communication Error", f"Failed to communicate with Rust server: {e}")
            return None
        except json.JSONDecodeError as e:
            messagebox.showerror("Data Error", f"Failed to parse data from Rust server: {e}")
            return None

# GUI Class to manage the interface
class ElevatorGUI:
    def __init__(self):
        """Initialize the GUI application."""
        # Initialize simulation client
        self.simulation_client = SimulationClient()

        # Get initial data from the server
        initial_state = self.simulation_client.send_request({"action": "get_state"})
        if initial_state is None:
            messagebox.showerror("Error", "Failed to get initial state from server.")
            sys.exit(1)

        # Extract necessary data
        self.elevators = initial_state.get('elevators', [])
        self.floors = initial_state.get('floors', {})
        self.simulation_time = initial_state.get('simulation_time', '00:00:00')
        self.floor_count = initial_state.get('floor_count', 10)
        self.elevator_count = initial_state.get('elevator_count', len(self.elevators))
        self.floor_heights = initial_state.get('floor_heights', [3] * (self.floor_count - 1))
        self.elevator_door_width = initial_state.get('elevator_door_width', 1.0)
        self.elevator_width = initial_state.get('elevator_width', 1.0)
        self.elevator_depth = initial_state.get('elevator_depth', 2.0)
        self.time_multiplier = initial_state.get('time_multiplier', 1.0)
        self.settings_saved = True  # Assuming settings are already saved from initial state

        # Initialize main window
        self.root = ctk.CTk()
        self.root.title("Asansör Simülasyonu")
        self.root.geometry("1600x900")
        ctk.set_appearance_mode("dark")
        ctk.set_default_color_theme("dark-blue")

        # Initialize variables
        self.canvas = None
        self.canvas_width = 800
        self.canvas_height = 800
        self.elevator_widgets = {}
        self.floor_labels = {}
        self.elevator_info_labels = {}
        self.floor_destination_labels = {}
        self.elevator_fill_bars = {}
        self.wait_time_history = []
        self.simulation_time_history = []
        self.max_wait_time = 0
        self.SIMULATION_RUNNING = False
        self.DOORS_OPEN = True

        # Initialize GUI elements
        self.create_menu()
        self.create_main_panes()
        self.create_control_frame()
        self.create_canvas()
        self.create_stats_frame()
        self.toggle_stats()
        self.initialize_visuals()

        # Start the GUI update loop
        self.root.after(1000, self.update_gui)
        self.root.mainloop()

    def create_menu(self):
        """Create the menu bar."""
        menu_bar = Menu(self.root)
        self.root.config(menu=menu_bar)

        # File menu
        file_menu = Menu(menu_bar, tearoff=0)
        file_menu.add_command(label="Çıkış", command=self.root.quit)
        menu_bar.add_cascade(label="Dosya", menu=file_menu)

        # View menu
        view_menu = Menu(menu_bar, tearoff=0)
        self.settings_visible = BooleanVar(value=True)
        self.controls_visible = BooleanVar(value=True)
        self.stats_visible = BooleanVar(value=True)
        self.time_skip_visible = BooleanVar(value=True)

        # Add checkbuttons to the View menu
        view_menu.add_checkbutton(label="Ayarları Göster", variable=self.settings_visible, command=self.toggle_settings)
        view_menu.add_checkbutton(label="Asansör Kontrollerini Göster", variable=self.controls_visible, command=self.toggle_controls)
        view_menu.add_checkbutton(label="İstatistikleri Göster", variable=self.stats_visible, command=self.toggle_stats)
        view_menu.add_checkbutton(label="Zaman Atlama Bölümünü Göster", variable=self.time_skip_visible, command=self.toggle_time_skip)
        menu_bar.add_cascade(label="Görünüm", menu=view_menu)

    def create_main_panes(self):
        """Create the main panes for the GUI."""
        # Main horizontal PanedWindow
        self.main_paned = ttk.PanedWindow(self.root, orient='horizontal')
        self.main_paned.pack(fill='both', expand=True)

        # Left frame (controls)
        self.left_frame = ctk.CTkFrame(self.main_paned, width=300)
        self.main_paned.add(self.left_frame)
        self.left_frame.pack_propagate(False)

        # Right vertical PanedWindow (canvas and stats)
        self.right_paned = ttk.PanedWindow(self.main_paned, orient='vertical')
        self.main_paned.add(self.right_paned, weight=3)  # Right pane is wider

        # Top frame (canvas)
        self.canvas_frame = ctk.CTkFrame(self.right_paned)
        self.right_paned.add(self.canvas_frame, weight=2)  # Elevator page is larger

        # Bottom frame (stats)
        self.stats_frame = ctk.CTkFrame(self.right_paned)
        # NOT: We manage stats_frame visibility with toggle_stats
        # self.right_paned.add(self.stats_frame, weight=1)  # Stats page is smaller

    def create_control_frame(self):
        """Create the control frame on the left."""
        self.control_frame = ctk.CTkScrollableFrame(self.left_frame)
        self.control_frame.pack(fill='both', expand=True)

        # Simulation control frame
        self.sim_control_frame = ctk.CTkFrame(self.control_frame)
        self.sim_control_frame.pack(fill='x', padx=5, pady=5)

        # Simulation switch
        self.sim_switch = ctk.CTkSwitch(self.sim_control_frame, text="Simülasyon", command=self.toggle_simulation)
        self.sim_switch.pack(side='left', padx=5)
        self.sim_switch.configure(state='normal')  # Enabled since settings are already saved

        # Door control switch
        self.door_switch = ctk.CTkSwitch(self.sim_control_frame, text="Kapılar Açık", command=self.toggle_doors)
        self.door_switch.select()  # Default is open
        self.door_switch.pack(side='left', padx=5)
        self.door_switch.configure(state='normal')  # Enabled

        # Simulation speed slider
        self.speed_frame = ctk.CTkFrame(self.control_frame)
        self.speed_frame.pack(fill='x', padx=5, pady=5)

        self.speed_label = ctk.CTkLabel(self.speed_frame, text="Simülasyon Hızı:")
        self.speed_label.pack(side='left', padx=5)

        self.speed_slider = ctk.CTkSlider(self.speed_frame, from_=1, to=10, number_of_steps=9, command=self.update_speed)
        self.speed_slider.set(self.time_multiplier)
        self.speed_slider.pack(side='left', fill='x', expand=True, padx=5)
        self.speed_slider.configure(state='normal')  # Enabled

        # Time skip buttons
        self.skip_frame = ctk.CTkFrame(self.control_frame)
        self.skip_frame.pack(fill='x', padx=5, pady=5)

        self.skip_label = ctk.CTkLabel(self.skip_frame, text="Zaman Atla:")
        self.skip_label.pack()

        self.button_frame = ctk.CTkFrame(self.skip_frame)
        self.button_frame.pack(fill='x')

        self.button_frame.columnconfigure((0, 1, 2), weight=1)
        self.morning_button = ctk.CTkButton(self.button_frame, text="Sabah", command=lambda: self.skip_to_time(6))
        self.midday_button = ctk.CTkButton(self.button_frame, text="Öğle", command=lambda: self.skip_to_time(12))
        self.evening_button = ctk.CTkButton(self.button_frame, text="Akşam", command=lambda: self.skip_to_time(17))

        self.morning_button.grid(row=0, column=0, sticky='nsew', padx=2, pady=2)
        self.midday_button.grid(row=0, column=1, sticky='nsew', padx=2, pady=2)
        self.evening_button.grid(row=0, column=2, sticky='nsew', padx=2, pady=2)

        # Settings frame
        self.settings_frame = ctk.CTkFrame(self.control_frame)
        self.settings_frame.pack(fill='x', padx=5, pady=5)
        self.create_settings_widgets()

        # Elevator manual control
        self.manual_control_frame = ctk.CTkFrame(self.control_frame)
        self.manual_control_frame.pack(fill='x', padx=5, pady=5)

        self.control_label = ctk.CTkLabel(self.manual_control_frame, text="Asansör Kontrolleri", font=("Arial", 16))
        self.control_label.pack(pady=5)
        self.create_elevator_controls()

        # Information message
        messagebox.showinfo("Bilgi", "Lütfen ayarları kontrol edin ve 'Ayarları Kaydet' butonuna basın.")

    def create_settings_widgets(self):
        """Create settings widgets in the settings frame."""
        # Clear previous widgets
        for widget in self.settings_frame.winfo_children():
            widget.destroy()

        settings_label = ctk.CTkLabel(self.settings_frame, text="Ayarlar", font=("Arial", 16))
        settings_label.pack(pady=5)

        # Floor count
        floor_count_frame = ctk.CTkFrame(self.settings_frame)
        floor_count_frame.pack(fill='x', padx=5, pady=2)
        floor_count_label = ctk.CTkLabel(floor_count_frame, text="Kat Sayısı:")
        floor_count_label.pack(side='left')
        self.floor_count_entry = ctk.CTkEntry(floor_count_frame)
        self.floor_count_entry.insert(0, str(self.floor_count))
        self.floor_count_entry.pack(side='left', fill='x', expand=True, padx=5)

        # Elevator count
        elevator_count_frame = ctk.CTkFrame(self.settings_frame)
        elevator_count_frame.pack(fill='x', padx=5, pady=2)
        elevator_count_label = ctk.CTkLabel(elevator_count_frame, text="Asansör Sayısı:")
        elevator_count_label.pack(side='left')
        self.elevator_count_entry = ctk.CTkEntry(elevator_count_frame)
        self.elevator_count_entry.insert(0, str(self.elevator_count))
        self.elevator_count_entry.pack(side='left', fill='x', expand=True, padx=5)

        # Elevator capacity
        elevator_capacity_frame = ctk.CTkFrame(self.settings_frame)
        elevator_capacity_frame.pack(fill='x', padx=5, pady=2)
        elevator_capacity_label = ctk.CTkLabel(elevator_capacity_frame, text="Asansör Kapasitesi (kg):")
        elevator_capacity_label.pack(side='left')
        self.elevator_capacity_entry = ctk.CTkEntry(elevator_capacity_frame)
        # Assuming each elevator has the same capacity
        default_capacity = self.elevators[0].get('capacity', 1000.0) if self.elevators else 1000.0
        self.elevator_capacity_entry.insert(0, str(default_capacity))
        self.elevator_capacity_entry.pack(side='left', fill='x', expand=True, padx=5)

        # Elevator door width
        elevator_door_width_frame = ctk.CTkFrame(self.settings_frame)
        elevator_door_width_frame.pack(fill='x', padx=5, pady=2)
        elevator_door_width_label = ctk.CTkLabel(elevator_door_width_frame, text="Asansör Kapı Genişliği (m):")
        elevator_door_width_label.pack(side='left')
        self.elevator_door_width_entry = ctk.CTkEntry(elevator_door_width_frame)
        # Assuming a default value; you can modify based on server data
        self.elevator_door_width_entry.insert(0, str(self.elevator_door_width))
        self.elevator_door_width_entry.pack(side='left', fill='x', expand=True, padx=5)

        # Elevator width
        elevator_width_frame = ctk.CTkFrame(self.settings_frame)
        elevator_width_frame.pack(fill='x', padx=5, pady=2)
        elevator_width_label = ctk.CTkLabel(elevator_width_frame, text="Asansör Genişliği (m):")
        elevator_width_label.pack(side='left')
        self.elevator_width_entry = ctk.CTkEntry(elevator_width_frame)
        self.elevator_width_entry.insert(0, str(self.elevator_width))
        self.elevator_width_entry.pack(side='left', fill='x', expand=True, padx=5)

        # Elevator depth
        elevator_depth_frame = ctk.CTkFrame(self.settings_frame)
        elevator_depth_frame.pack(fill='x', padx=5, pady=2)
        elevator_depth_label = ctk.CTkLabel(elevator_depth_frame, text="Asansör Derinliği (m):")
        elevator_depth_label.pack(side='left')
        self.elevator_depth_entry = ctk.CTkEntry(elevator_depth_frame)
        self.elevator_depth_entry.insert(0, str(self.elevator_depth))
        self.elevator_depth_entry.pack(side='left', fill='x', expand=True, padx=5)

        # Floor heights
        floor_heights_label = ctk.CTkLabel(self.settings_frame, text="Katlar Arası Yükseklikler (m):")
        floor_heights_label.pack(pady=5)

        self.floor_height_entries = []
        for i in range(self.floor_count - 1):
            floor_height_frame = ctk.CTkFrame(self.settings_frame)
            floor_height_frame.pack(fill='x', padx=5, pady=2)
            label = ctk.CTkLabel(floor_height_frame, text=f"{i+1}-{i+2} Kat:")
            label.pack(side='left')
            entry = ctk.CTkEntry(floor_height_frame)
            if i < len(self.floor_heights):
                entry.insert(0, str(self.floor_heights[i]))
            else:
                entry.insert(0, '3')  # Default 3 meters
            entry.pack(side='left', fill='x', expand=True, padx=5)
            self.floor_height_entries.append(entry)

        # Save settings button
        save_settings_button = ctk.CTkButton(self.settings_frame, text="Ayarları Kaydet", command=self.save_settings)
        save_settings_button.pack(fill='x', padx=5, pady=5)

    def create_elevator_controls(self):
        """Create elevator manual controls."""
        # Clear previous controls
        for widget in self.manual_control_frame.winfo_children():
            widget.destroy()

        # Create new controls for each elevator
        for elevator in self.elevators:
            elevator_frame = ctk.CTkFrame(self.manual_control_frame)
            elevator_frame.pack(fill='x', padx=5, pady=5)
            elevator_label = ctk.CTkLabel(elevator_frame, text=f"Asansör {elevator['id']}")
            elevator_label.pack(side='left')

            # Floor selection combobox
            floor_var = ctk.StringVar(value=str(1))
            floor_combobox = ctk.CTkComboBox(
                elevator_frame,
                values=[str(i) for i in range(1, self.floor_count + 1)],
                variable=floor_var
            )
            floor_combobox.pack(side='left', fill='x', expand=True, padx=5)

            # Go button to set manual target
            go_button = ctk.CTkButton(
                elevator_frame,
                text="Git",
                command=lambda e_id=elevator['id'], fv=floor_var: self.set_manual_target(e_id, int(fv.get()))
            )
            go_button.pack(side='left')

    def create_canvas(self):
        """Create the canvas for elevators and floors."""
        # Create the canvas inside canvas_frame
        self.canvas = Canvas(self.canvas_frame, bg='#2b2b2b', highlightthickness=0)
        self.canvas.pack(fill='both', expand=True, side='left')

        # Scrollbar for canvas
        self.canvas_scrollbar = ttk.Scrollbar(self.canvas_frame, orient='vertical', command=self.canvas.yview)
        self.canvas_scrollbar.pack(side='right', fill='y')
        self.canvas.configure(yscrollcommand=self.canvas_scrollbar.set)

        # Mouse scroll for canvas
        def on_mousewheel(event):
            self.canvas.yview_scroll(int(-1 * (event.delta / 120)), "units")

        self.canvas.bind_all("<MouseWheel>", on_mousewheel)

    def create_stats_frame(self):
        """Create the statistics frame with Treeview and Matplotlib plot."""
        # Create a PanedWindow inside stats_frame to make left and right frames resizable
        self.stats_paned = ttk.PanedWindow(self.stats_frame, orient='horizontal')
        self.stats_paned.pack(fill='both', expand=True)

        # Left frame for Treeview
        self.stats_left_frame = ctk.CTkFrame(self.stats_paned)
        self.stats_paned.add(self.stats_left_frame, weight=1)  # Treeview

        # Right frame for Matplotlib plot
        self.stats_right_frame = ctk.CTkFrame(self.stats_paned)
        self.stats_paned.add(self.stats_right_frame, weight=3)  # Matplotlib plot

        # Treeview style configuration
        style = ttk.Style()
        style.theme_use("clam")  # Use 'clam' theme for better customization

        # Configure Treeview appearance
        style.configure("Custom.Treeview",
                        foreground="white",            # Text color
                        background="#2b2b2b",          # Cell background color
                        fieldbackground="#2b2b2b",     # Field background color
                        rowheight=25,                  # Row height
                        borderwidth=0,                 # Border width
                        highlightthickness=0)          # Highlight thickness

        # Configure selected row appearance
        style.map("Custom.Treeview",
                  background=[('selected', '#4a90e2')],  # Selected row background color
                  foreground=[('selected', 'white')])    # Selected row text color

        # Configure Treeview heading appearance
        style.configure("Custom.Treeview.Heading",
                        foreground="white",            # Heading text color
                        background="#2b2b2b",          # Heading background color
                        borderwidth=0,                 # Heading border width
                        relief="flat")                 # Heading relief

        # Configure active heading appearance
        style.map("Custom.Treeview.Heading",
                  foreground=[('active', 'white')],
                  background=[('active', '#2b2b2b')])

        # Create Treeview
        self.tree = ttk.Treeview(self.stats_left_frame, style="Custom.Treeview")
        self.tree.pack(fill='both', expand=True)

        # Define columns
        self.tree['columns'] = ("value",)
        self.tree.column("#0", width=200, minwidth=150, anchor='w')    # First column
        self.tree.column("value", width=200, minwidth=150, anchor='w')  # Value column
        self.tree.heading("#0", text="İstatistik", anchor='w')         # First column heading
        self.tree.heading("value", text="Değer", anchor='w')           # Value column heading

        # Add grouped statistics
        self.tree.insert("", "end", "waiting_times", text="Bekleme Süresi", open=True)
        self.tree.insert("waiting_times", "end", "average_wait", text="Ortalama Bekleme Süresi", values=("0 sn",))
        self.tree.insert("waiting_times", "end", "max_wait", text="Maksimum Bekleme Süresi", values=("0 sn",))

        self.tree.insert("", "end", "journey_times", text="Yolculuk Süresi", open=True)
        self.tree.insert("journey_times", "end", "average_journey", text="Ortalama Yolculuk Süresi", values=("0 sn",))

        self.tree.insert("", "end", "passenger_counts", text="Yolcu Sayısı", open=True)
        self.tree.insert("passenger_counts", "end", "total_served", text="Hizmet Edilen Yolcu Sayısı", values=("0",))

        self.tree.insert("", "end", "floor_distribution", text="Kat Dağılımı", open=True)
        # Add floor distribution dynamically
        for floor in range(1, self.floor_count + 1):
            self.tree.insert("floor_distribution", "end", f"floor_{floor}", text=f"Kat {floor}", values=("Bekleyen 0, Merdiven 0",))

        # Matplotlib graph for statistics
        self.fig, self.ax = plt.subplots(figsize=(6, 4))
        plt.style.use('dark_background')                           # Use dark theme
        self.ax.set_facecolor('#2b2b2b')                           # Set graph background color
        self.fig.patch.set_facecolor('#2b2b2b')                    # Set figure background color
        self.ax.set_title('Ortalama Bekleme Süresi Zaman İçinde', color='white')  # Title
        self.ax.set_xlabel('Simülasyon Zamanı', color='white')     # X-axis label
        self.ax.set_ylabel('Ortalama Bekleme Süresi (sn)', color='white')  # Y-axis label
        self.ax.tick_params(axis='x', colors='white')              # X-axis tick colors
        self.ax.tick_params(axis='y', colors='white')              # Y-axis tick colors
        self.ax.grid(True, color='gray')                           # Grid lines color
        self.line, = self.ax.plot([], [], marker='o', linestyle='-', color='white')  # Line color

        # Embed Matplotlib plot in Tkinter
        self.canvas_plot = FigureCanvasTkAgg(self.fig, master=self.stats_right_frame)
        self.canvas_plot.draw()
        self.canvas_plot.get_tk_widget().pack(fill='both', expand=True)

    def initialize_visuals(self):
        """Initialize visuals on the canvas."""
        # Clear canvas
        self.canvas.delete("all")

        # Draw floors and labels
        total_height = sum([height for height in self.floor_heights])
        pixel_per_meter = 50  # Scaling factor
        y_positions = [0]
        for height in self.floor_heights:
            y_positions.append(y_positions[-1] + height * pixel_per_meter)
        self.canvas_height = y_positions[-1]
        self.canvas.config(scrollregion=(0, 0, self.canvas_width, self.canvas_height))
        self.floor_labels = {}
        self.floor_destination_labels = {}

        for i in range(len(y_positions)):
            y = self.canvas_height - y_positions[i]
            # Draw floor lines
            self.canvas.create_line(0, y, self.canvas_width, y, fill="white")

            # Calculate mid-point for labels
            if i < len(y_positions) - 1:
                y_mid = y - (y_positions[i+1] - y_positions[i]) / 2
            else:
                if len(y_positions) > 1:
                    y_mid = y + (y_positions[i] - y_positions[i-1]) / 2
                else:
                    y_mid = y
            # Floor label on the left
            label = self.canvas.create_text(10, y_mid, anchor='w', text=f"Kat {i+1}", fill="white")
            self.floor_labels[i+1] = label
            # Destination label on the right
            destination_label = self.canvas.create_text(
                self.canvas_width - 10, y_mid,
                anchor='e', text="", width=200, justify='right', fill="white"
            )
            self.floor_destination_labels[i+1] = destination_label

        # Draw elevators
        self.elevator_widgets = {}
        self.elevator_info_labels = {}
        self.elevator_fill_bars = {}
        elevator_width_visual = 50  # Visual elevator width
        elevator_height_visual = 40  # Visual elevator height
        elevator_spacing = self.canvas_width / (self.elevator_count + 1)
        for i, elevator in enumerate(self.elevators):
            x = elevator_spacing * (i + 1)
            # Initial position
            y1 = self.canvas_height - elevator['current_height'] * pixel_per_meter - 10
            y2 = y1 - elevator_height_visual

            # Elevator rectangle
            elevator_rect = self.canvas.create_rectangle(
                x - elevator_width_visual / 2,
                y1,
                x + elevator_width_visual / 2,
                y2,
                fill='gray'
            )
            self.elevator_widgets[elevator['id']] = elevator_rect

            # Fill bar to represent capacity
            fill_bar = self.canvas.create_rectangle(
                x - elevator_width_visual / 2,
                y1,
                x + elevator_width_visual / 2,
                y1,  # Initially zero fill
                fill='green'
            )
            self.elevator_fill_bars[elevator['id']] = fill_bar

            # Elevator info label
            info_label = self.canvas.create_text(
                x,
                y2 - 30,
                text=f"Asansör {elevator['id']}: Beklemede\nToplam Ağırlık: 0 kg",
                font=('Arial', 10), fill="white"
            )
            self.elevator_info_labels[elevator['id']] = info_label

    def update_gui(self):
        """Update GUI elements periodically."""
        # Request the current state from the Rust server
        request = {"action": "get_state"}
        response = self.simulation_client.send_request(request)
        if response is None:
            self.root.after(1000, self.update_gui)
            return

        # Parse the response and update the GUI
        self.elevators = response.get('elevators', [])
        self.floors = response.get('floors', {})
        self.simulation_time = response.get('simulation_time', '00:00:00')

        # Update elevators
        self.update_elevators()

        # Update floors
        self.update_floors()

        # Update statistics or other GUI elements as needed
        self.update_statistics()

        # Schedule next update
        self.root.after(1000, self.update_gui)

    def update_elevators(self):
        """Update elevators based on data from the server."""
        total_height = sum([height for height in self.floor_heights])
        pixel_per_meter = 50
        y_positions = [0]
        for height in self.floor_heights:
            y_positions.append(y_positions[-1] + height * pixel_per_meter)
        self.canvas_height = y_positions[-1]

        for elevator in self.elevators:
            rect = self.elevator_widgets.get(elevator['id'])
            fill_bar = self.elevator_fill_bars.get(elevator['id'])
            if rect is None or fill_bar is None:
                continue
            x1, y1, x2, y2 = self.canvas.coords(rect)
            x_center = (x1 + x2) / 2

            # Update elevator position
            y1 = self.canvas_height - elevator['current_height'] * pixel_per_meter - 10
            y2 = y1 - 40  # Visual height of the elevator
            self.canvas.coords(rect, x1, y1, x2, y2)

            # Update elevator color based on movement
            fill_color = 'gray' if not elevator['moving'] else 'green'
            self.canvas.itemconfig(rect, fill=fill_color)

            # Calculate total weight and fill percentage
            total_weight = sum([p['weight'] * p['group_size'] for p in elevator['passengers']])
            fill_percentage = total_weight / elevator['capacity']
            fill_percentage = min(fill_percentage, 1.0)  # Cap at 100%

            # Update fill bar height and color
            fill_height = (y1 - y2) * fill_percentage
            self.canvas.coords(
                fill_bar,
                x1,
                y1,
                x2,
                y1 - fill_height
            )
            # Change color based on fill percentage
            if fill_percentage <= 0.7:
                bar_color = 'green'
            elif fill_percentage <= 0.9:
                bar_color = 'yellow'
            else:
                bar_color = 'red'
            self.canvas.itemconfig(fill_bar, fill=bar_color)

            # Update elevator info label
            info_label = self.elevator_info_labels.get(elevator['id'])
            if info_label is None:
                continue
            # Passengers' destination floors
            destination_counts = {}
            for passenger in elevator['passengers']:
                dest_floor = passenger['destination_floor']
                destination_counts[dest_floor] = destination_counts.get(dest_floor, 0) + passenger['group_size']

            dest_text = ', '.join([f"{count}K {floor}.K" for floor, count in destination_counts.items()])
            status_text = f"{dest_text}" if dest_text else "Boş"

            passenger_count = sum(p['group_size'] for p in elevator['passengers'])
            disabled_count = sum(p['group_size'] for p in elevator['passengers'] if p['is_disabled'])
            info_text = (
                f"Asansör {elevator['id']}: {status_text}\n"
                f"Toplam Ağırlık: {total_weight:.2f} kg\n"
                f"Yolcu Sayısı: {passenger_count}\n"
                f"Engelli Yolcu: {disabled_count}"
            )
            self.canvas.coords(info_label, x_center, y2 - 30)
            self.canvas.itemconfig(info_label, text=info_text)

    def update_floors(self):
        """Update floor labels and waiting passenger info."""
        total_height = sum([height for height in self.floor_heights])
        pixel_per_meter = 50
        y_positions = [0]
        for height in self.floor_heights:
            y_positions.append(y_positions[-1] + height * pixel_per_meter)
        self.canvas_height = y_positions[-1]

        for i in range(len(y_positions)):
            y = self.canvas_height - y_positions[i]
            label = self.floor_labels.get(i+1)
            if label:
                floor_data = self.floors.get(str(i+1), {'waiting': [], 'stairs': []})
                waiting_passengers = floor_data.get('waiting', [])
                stairs_count = len(floor_data.get('stairs', []))
                waiting_count = sum(p['group_size'] for p in waiting_passengers)
                label_text = f"Kat {i+1} - Bekleyen: {waiting_count}, Merdiven: {stairs_count}"
                self.canvas.itemconfig(label, text=label_text)
                if i < len(y_positions) - 1:
                    y_mid = y - (y_positions[i+1] - y_positions[i]) / 2
                else:
                    y_mid = y + 20
                self.canvas.coords(label, 10, y_mid)

                # Waiting passengers' destination floors
                destination_counts = {}
                for passenger in waiting_passengers:
                    dest_floor = passenger['destination_floor']
                    destination_counts[dest_floor] = destination_counts.get(dest_floor, 0) + passenger['group_size']
                dest_text = '\n'.join([f"{dest}: {count}" for dest, count in destination_counts.items()])
                dest_label = self.floor_destination_labels.get(i+1)
                self.canvas.itemconfig(dest_label, text=dest_text)
                self.canvas.coords(dest_label, self.canvas_width - 10, y_mid)

    def update_statistics(self):
        """Update statistics displayed in the GUI."""
        # Request statistics from the server
        request = {"action": "get_statistics"}
        response = self.simulation_client.send_request(request)
        if response is None:
            return

        average_wait = response.get('average_wait_time', 0)
        max_wait_time = response.get('max_wait_time', 0)
        total_served_passengers = response.get('total_served_passengers', 0)
        average_journey_time = response.get('average_journey_time', 0)

        # Update Treeview
        self.tree.set("average_wait", column="value", value=f"{average_wait:.2f} sn")
        self.tree.set("max_wait", column="value", value=f"{max_wait_time:.2f} sn")
        self.tree.set("average_journey", column="value", value=f"{average_journey_time:.2f} sn")
        self.tree.set("total_served", column="value", value=f"{total_served_passengers}")

        # Update floor distribution
        for floor in range(1, self.floor_count + 1):
            floor_data = self.floors.get(str(floor), {'waiting': [], 'stairs': []})
            waiting_passengers = floor_data.get('waiting', [])
            stairs_count = len(floor_data.get('stairs', []))
            waiting_count = sum(p['group_size'] for p in waiting_passengers)
            self.tree.set(f"floor_{floor}", column="value", value=f"Bekleyen {waiting_count}, Merdiven {stairs_count}")

        # Update plot data
        self.wait_time_history.append(average_wait)
        self.simulation_time_history.append(self.simulation_time)

        # Update the matplotlib plot
        self.ax.clear()
        plt.style.use('dark_background')
        self.ax.set_facecolor('#2b2b2b')  # Set graph background color
        self.fig.patch.set_facecolor('#2b2b2b')  # Set figure background color
        self.ax.set_title('Ortalama Bekleme Süresi Zaman İçinde', color='white')
        self.ax.set_xlabel('Simülasyon Zamanı', color='white')
        self.ax.set_ylabel('Ortalama Bekleme Süresi (sn)', color='white')
        self.ax.tick_params(axis='x', colors='white')  # X-axis tick colors
        self.ax.tick_params(axis='y', colors='white')  # Y-axis tick colors
        self.ax.grid(True, color='gray')  # Grid lines color
        self.ax.plot(self.simulation_time_history, self.wait_time_history, marker='o', linestyle='-', color='white')  # Line color
        self.fig.tight_layout()
        self.canvas_plot.draw()

    def toggle_simulation(self):
        """Toggle simulation running state."""
        if not self.settings_saved:
            messagebox.showwarning("Uyarı", "Lütfen ayarları kaydetmek için 'Ayarları Kaydet' butonuna basın.")
            self.sim_switch.deselect()
            return
        if self.sim_switch.get():
            self.SIMULATION_RUNNING = True
            # Notify server to start simulation
            request = {"action": "start_simulation"}
            response = self.simulation_client.send_request(request)
            if response and response.get('success', False):
                messagebox.showinfo("Başarılı", response.get('message', 'Simülasyon başlatıldı.'))
            else:
                messagebox.showerror("Hata", response.get('message', 'Simülasyon başlatılamadı.'))
        else:
            self.SIMULATION_RUNNING = False
            # Notify server to stop simulation
            request = {"action": "stop_simulation"}
            response = self.simulation_client.send_request(request)
            if response and response.get('success', False):
                messagebox.showinfo("Başarılı", response.get('message', 'Simülasyon durduruldu.'))
            else:
                messagebox.showerror("Hata", response.get('message', 'Simülasyon durdurulamadı.'))

    def toggle_doors(self):
        """Toggle doors open/close."""
        if not self.settings_saved:
            messagebox.showwarning("Uyarı", "Lütfen ayarları kaydetmek için 'Ayarları Kaydet' butonuna basın.")
            self.door_switch.select()
            return
        self.DOORS_OPEN = self.door_switch.get()
        # Notify server about door state
        request = {"action": "toggle_doors", "doors_open": self.DOORS_OPEN}
        response = self.simulation_client.send_request(request)
        if response and response.get('success', False):
            state = "açık" if self.DOORS_OPEN else "kapalı"
            messagebox.showinfo("Başarılı", f"Kapılar {state} oldu.")
        else:
            messagebox.showerror("Hata", response.get('message', 'Kapılar kontrol edilemedi.'))

    def update_speed(self, val):
        """Update simulation speed."""
        if not self.settings_saved:
            messagebox.showwarning("Uyarı", "Lütfen ayarları kaydetmek için 'Ayarları Kaydet' butonuna basın.")
            self.speed_slider.set(1)
            return
        self.time_multiplier = float(val)
        # Notify server about speed change
        request = {"action": "update_speed", "time_multiplier": self.time_multiplier}
        response = self.simulation_client.send_request(request)
        if response and response.get('success', False):
            messagebox.showinfo("Başarılı", f"Simülasyon hızı {self.time_multiplier}x olarak ayarlandı.")
        else:
            messagebox.showerror("Hata", response.get('message', 'Simülasyon hızı güncellenemedi.'))

    def skip_to_time(self, hour):
        """Skip to a specific time."""
        if not self.settings_saved:
            messagebox.showwarning("Uyarı", "Lütfen ayarları kaydetmek için 'Ayarları Kaydet' butonuna basın.")
            return
        try:
            # Ensure hour is within 0-23
            if not (0 <= hour <= 23):
                raise ValueError("Saat 0-23 arasında olmalıdır.")
            new_time_str = f"{hour:02d}:00:00"
            # Notify server about time skip
            request = {"action": "skip_to_time", "new_time": new_time_str}
            response = self.simulation_client.send_request(request)
            if response and response.get('success', False):
                self.simulation_time = new_time_str
                messagebox.showinfo("Başarılı", f"Simülasyon saati {new_time_str} olarak ayarlandı.")
            else:
                messagebox.showerror("Hata", response.get('message', 'Simülasyon saati ayarlanamadı.'))
        except ValueError as ve:
            messagebox.showerror("Hata", f"Geçersiz saat değeri: {ve}")

    def save_settings(self):
        """Save settings and update simulation."""
        try:
            new_floor_count = int(self.floor_count_entry.get())
            new_elevator_count = int(self.elevator_count_entry.get())
            new_elevator_capacity = float(self.elevator_capacity_entry.get())
            new_elevator_door_width = float(self.elevator_door_width_entry.get())
            new_elevator_width = float(self.elevator_width_entry.get())
            new_elevator_depth = float(self.elevator_depth_entry.get())
            new_floor_heights = [float(entry.get()) for entry in self.floor_height_entries]
        except ValueError:
            messagebox.showerror("Hatalı Giriş", "Lütfen tüm ayarları doğru formatta girin.")
            return

        new_settings = {
            'floor_count': new_floor_count,
            'elevator_count': new_elevator_count,
            'elevator_capacity': new_elevator_capacity,
            'elevator_door_width': new_elevator_door_width,
            'elevator_width': new_elevator_width,
            'elevator_depth': new_elevator_depth,
            'floor_heights': new_floor_heights
        }

        # Send settings to the server
        request = {"action": "update_settings", "settings": new_settings}
        response = self.simulation_client.send_request(request)

        if response and response.get('success', False):
            # Update local variables based on new settings
            self.floor_count = new_floor_count
            self.elevator_count = new_elevator_count
            self.floor_heights = new_floor_heights

            # Update elevators list if elevator count has changed
            if len(self.elevators) != new_elevator_count:
                # Request updated elevators from the server
                request = {"action": "get_elevators"}
                elevators_response = self.simulation_client.send_request(request)
                if elevators_response and 'elevators' in elevators_response:
                    self.elevators = elevators_response['elevators']
                    self.create_elevator_controls()
                    self.initialize_visuals()

            # Re-initialize visuals
            self.initialize_visuals()
            self.create_elevator_controls()
            messagebox.showinfo("Başarılı", response.get('message', 'Ayarlar kaydedildi.'))
        else:
            messagebox.showerror("Hatalı Giriş", response.get('message', 'Ayarlar kaydedilemedi.'))

    def toggle_settings(self):
        """Toggle visibility of settings."""
        if self.settings_visible.get():
            self.settings_frame.pack(fill='x', padx=5, pady=5)
        else:
            self.settings_frame.pack_forget()

    def toggle_controls(self):
        """Toggle visibility of controls."""
        if self.controls_visible.get():
            self.manual_control_frame.pack(fill='x', padx=5, pady=5)
        else:
            self.manual_control_frame.pack_forget()

    def toggle_stats(self):
        """Toggle visibility of statistics."""
        if self.stats_visible.get():
            # Add stats_frame back to right_paned if not already present
            if self.stats_frame not in self.right_paned.panes():
                self.right_paned.add(self.stats_frame, weight=1)
        else:
            # Remove stats_frame from right_paned if present
            if self.stats_frame in self.right_paned.panes():
                self.right_paned.forget(self.stats_frame)

    def set_manual_target(self, elevator_id, target_floor):
        """Send a request to set manual target for an elevator."""
        request = {
            "action": "set_manual_target",
            "elevator_id": elevator_id,
            "target_floor": target_floor
        }
        response = self.simulation_client.send_request(request)
        if response and response.get('success', False):
            messagebox.showinfo("Başarılı", response.get('message', f"Asansör {elevator_id} {target_floor}. kata gönderildi."))
        else:
            messagebox.showerror("Hata", response.get('message', f"Asansör {elevator_id} hedef kata gönderilemedi."))

# Run the GUI application
if __name__ == "__main__":
    # Start the GUI
    app = ElevatorGUI()
