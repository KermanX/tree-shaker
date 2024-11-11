// @react-jsx

function Comp({ x, y, z }) {
  const unused = <div b={effect()} />;

  return (
    <div>
      {x} {y} {z}
    </div>
  );
}

export function main() {
  return (
    <Comp x={1} y>
      <div id="1" />
      Hello
    </Comp>
  );
}
