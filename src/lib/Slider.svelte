<script lang="ts">
    export let name: string;
    export let min: number;
    export let max: number;
    export let step: number;
    export let value: number;
    export let onChange: (value: number) => void;
</script>

<div>
    <h2>{name}</h2>
    <input
        type="range"
        bind:value={value}
        min={min}
        max={max}
        step={step}
        on:input={() => onChange(value)}
    />
    <output>{value}</output>
</div>
