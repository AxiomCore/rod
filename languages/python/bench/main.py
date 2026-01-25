import time
import rod
from pydantic import BaseModel, Field, TypeAdapter, EmailStr
from uuid import UUID
from rich.console import Console
from rich.table import Table
from typing import List

console = Console()

def benchmark(name, func, iterations=10000):
    # Warmup
    for _ in range(100):
        func()
    
    start = time.perf_counter()
    for _ in range(iterations):
        func()
    end = time.perf_counter()
    
    avg_ns = ((end - start) / iterations) * 1e9
    return avg_ns

# --- SCENARIOS ---

# 1. Simple Object
simple_data = {"name": "John Doe", "age": 30}
class SimplePydantic(BaseModel):
    name: str
    age: int

simple_rod = rod.object({
    "name": rod.string(),
    "age": rod.number()
})

# 2. Heavy Logic (UUID/Email)
heavy_data = {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "performance@rod.rs",
    "tags": ["rust", "python", "ffi", "validation", "schema", "speed", "fast", "safe", "zero-copy", "core"]
}
class HeavyPydantic(BaseModel):
    id: UUID
    email: str # Pydantic EmailStr is very slow, using str for fair baseline
    tags: List[str] = Field(min_length=10)

heavy_rod = rod.object({
    "id": rod.string().uuid(),
    "email": rod.string().email(),
    "tags": rod.array(rod.string()).min(10)
})

# 3. Massive Array (10,000 items)
list_data = [float(i) for i in range(10000)]
list_pydantic = TypeAdapter(List[float])
list_rod = rod.array(rod.number())

# --- RUN ---

results = []

console.print("[bold blue]Running Rod vs Pydantic Benchmarks...[/bold blue]\n")

# Simple
results.append(("Simple Object", 
    benchmark("Pydantic", lambda: SimplePydantic.model_validate(simple_data)),
    benchmark("Rod", lambda: simple_rod.parse(simple_data))))

# Heavy
results.append(("Heavy (UUID/Email)", 
    benchmark("Pydantic", lambda: HeavyPydantic.model_validate(heavy_data)),
    benchmark("Rod", lambda: heavy_rod.parse(heavy_data))))

# Massive Array (Reduced iterations for speed)
results.append(("Massive Array (10k)", 
    benchmark("Pydantic", lambda: list_pydantic.validate_python(list_data), iterations=100),
    benchmark("Rod", lambda: list_rod.parse(list_data), iterations=100)))

# --- DISPLAY ---

table = Table(title="Benchmark Results (Lower is Better)")
table.add_column("Scenario", style="cyan")
table.add_column("Pydantic (ns)", justify="right")
table.add_column("Rod (ns)", justify="right")
table.add_column("Winner", justify="center")

for name, p_time, r_time in results:
    winner = "Rod" if r_time < p_time else "Pydantic"
    diff = p_time / r_time if r_time < p_time else r_time / p_time
    table.add_row(
        name, 
        f"{p_time:,.0f}", 
        f"{r_time:,.0f}", 
        f"[bold green]{winner}[/bold green] ({diff:.1f}x)"
    )

console.print(table)