export const DEMO = `export function evaluation() {
  function g(a) {
    if (a)
      console.log('effect')
    else
      return 'str'
  }
  let { ["x"]: y = 1 } = { x: g('') ? undefined : g(1) }
  return y
}

function Name({ name, info }) {
  const unused = <div b={effect()} />;

  return (
    <span>
      {name}
      {info && <sub> Lots of things never rendered </sub>}
    </span>
  );
}

export function Main() {
  return (
    <div>
      Hello
      <Name name={"world"} />
    </div>
  );
}
`