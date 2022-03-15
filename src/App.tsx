import { useEffect, useState } from "react";
import init, { js_generate } from "agar-man";
import { stringify } from "postcss";

const Poem: React.VFC<{ seed: string; sentence: string[] }> = ({
  seed,
  sentence,
}) => {
  const charPositions: { [key: string]: number[] } = {};

  seed.split("").forEach((letter, index) => {
    if (letter in charPositions) {
      charPositions[letter].push(index);
    } else {
      charPositions[letter] = [index];
    }
  });

  console.log(seed)
  console.log(sentence)

  return (
    <div className="flex">
      {sentence.map((word) => (
        <div className="ml-4 flex" key={word}>
          {word.split("").map((letter, index) => {
            const letterIndex = charPositions[letter].shift()!;
            const offset = seed.length - letterIndex;
            const filler = new Array(offset).fill("x");
            return (
              <div className="flex flex-col" key={index}>
                {filler.map((x, index) => {
                  return (
                    <div key={index} className="leading-5">
                      &nbsp;
                    </div>
                  );
                })}
                {seed.split("").map((letter, index) => {
                  return (
                    <div
                      key={index}
                      className={`w-6 leading-5 text-center ${
                        index !== letterIndex && "text-gray-300"
                      }`}
                    >
                      {letter === " " ? <span>&nbsp;</span> : letter}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      ))}
    </div>
  );
};

function App() {
  const [seed, setSeed] = useState("anagram");
  const [results, setResults] = useState<string[]>([]);

  const [rendered, setRendered] = useState<{seed: string, sentence:string[]}>({
    seed: "anagram",
    sentence: ["agar", "man"],
  })

  const [minLength, setMinLength] = useState<number | null>(3);
  const [executionTime, setExecutionTime] = useState(0);
  const [lengthOptions, setLengthOptions] = useState<number[]>([]);

  useEffect(() => {
    generate();
  }, []);

  const generate = () => {
    init().then(() => {
      var start = window.performance.now();
      let r = js_generate(seed, minLength || 5);
      setResults([...r]);
      var end = window.performance.now();
      setExecutionTime(Math.floor(end - start));
    });
  };

  return (
    <div className="flex h-screen">
      <div className="w-1/2 sm:w-1/3 md:w-1/4 lg:w-1/6 border-r h-screen flex flex-col">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            generate();
          }}
        >
          <input
            className="border-b w-full px-2 py-1"
            type="text"
            value={seed}
            onChange={(e) => {
              const newLengthOptions = [];

              const min = Math.max(
                2,
                Math.ceil(Math.sqrt(e.target.value.length))
              );

              for (
                let i = min;
                i < Math.ceil(e.target.value.length / 2) + 1;
                i++
              ) {
                newLengthOptions.push(Math.floor(i));
              }

              setLengthOptions(newLengthOptions);

              if (minLength === null || minLength === undefined) {
                setMinLength(newLengthOptions[0]);
              } else {
                if (minLength < newLengthOptions[0]) {
                  setMinLength(newLengthOptions[0]);
                }
                if (minLength > newLengthOptions[newLengthOptions.length - 1]) {
                  setMinLength(newLengthOptions[newLengthOptions.length - 1]);
                }
              }

              setSeed(e.target.value);
            }}
          />
        </form>
        <div className="flex w-full">
          {lengthOptions.map((length, index) => {
            return (
              <button
                key={length}
                onClick={() => setMinLength(length)}
                className={`px-2 py-1 text-sm border-r ${
                  length === minLength && "bg-black text-white"
                }`}
              >
                {length} {index === lengthOptions.length - 1 && "+"}
              </button>
            );
          })}
        </div>

        <div className="text-gray-400 px-2 pt-1 border-b">
          {results.length} results in {executionTime}ms
        </div>

        <div className="overflow-y-scroll">
          {results.map((result) => {
            const split_result = result.split("|");

            return (
              <button
                key={result}
                className="flex px-2 hover:bg-gray-100 w-full"
                onClick={() => {
                  setRendered({seed,sentence:split_result});
                }}
              >
                {split_result.map((word) => (
                  <div key={word} className="mr-2">
                    {word}
                  </div>
                ))}
              </button>
            );
          })}
        </div>
      </div>
      <div>
        {rendered !== null && (
          <Poem seed={rendered.seed} sentence={rendered.sentence} />
        )}
      </div>
    </div>
  );
}

export default App;
