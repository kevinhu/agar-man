import { Tab } from "@headlessui/react";
import init, { js_generate } from "agar-man";
import { Fragment, useEffect, useState } from "react";
import { BsArrowRightShort } from "react-icons/bs";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeList as List } from "react-window";
import { Poem } from "../components/Poem";

interface Rendered {
  seed: string;
  sentence: string[];
}

const Results: React.VFC<{
  results: string[];
  setRendered: React.Dispatch<React.SetStateAction<Rendered>>;
  renderedSeed: string;
}> = ({ results, setRendered, renderedSeed }) => {
  return (
    <div className="h-screen overflow-y-scroll">
      <AutoSizer>
        {({ height, width }) => (
          <List
            className="List"
            height={height}
            itemCount={results.length}
            itemSize={22}
            width={width}
            itemData={results}
          >
            {({ data, index, style }) => {
              const result = data[index];
              const split_result = result.split("|");

              return (
                <button
                  key={result}
                  className="flex w-full px-2 hover:bg-gray-100"
                  onClick={() => {
                    setRendered({ seed: renderedSeed, sentence: split_result });
                  }}
                  style={style}
                >
                  {split_result.map((word, index) => (
                    <div key={index} className="mr-2">
                      {word}
                    </div>
                  ))}
                </button>
              );
            }}
          </List>
        )}
      </AutoSizer>
    </div>
  );
};

const Input: React.VFC<{
  loading: boolean;
  generate: ({ seed, minLength }: { seed: string; minLength: number }) => void;
}> = ({ loading, generate }) => {
  const [seed, setSeed] = useState("anagram");

  const [minLength, setMinLength] = useState<number | null>(3);
  const [lengthOptions, setLengthOptions] = useState<number[]>([]);

  return (
    <>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          generate({ seed, minLength: minLength || 5 });
        }}
        className="flex"
      >
        <input
          spellCheck="false"
          autoCapitalize="none"
          autoCorrect="off"
          autoComplete="off"
          className="w-full px-2 py-1 border-b border-black outline-none"
          placeholder="Starter word..."
          type="text"
          value={seed}
          disabled={loading}
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
        <button
          type="submit"
          className="px-2 border-b border-l border-black hover:bg-gray-100"
          disabled={loading}
        >
          <BsArrowRightShort />
        </button>
      </form>
      {lengthOptions.length > 0 && (
        <div className="flex flex-wrap w-full -mt-px text-sm border-b border-black">
          <div className="px-2 py-1">Min len</div>
          {lengthOptions.map((length, index) => {
            return (
              <button
                key={length}
                onClick={() => {
                  setMinLength(length);
                  generate({ seed, minLength: length });
                }}
                className={`px-2 py-1 text-sm border-r border-b border-t -mb-px border-black ${
                  length === minLength && "bg-black text-white"
                }`}
              >
                {length} {index === lengthOptions.length - 1 && "+"}
              </button>
            );
          })}
        </div>
      )}
    </>
  );
};

export const Search = () => {
  const [renderedSeed, setRenderedSeed] = useState("anagram");

  const [loading, setLoading] = useState(false);

  const [results, setResults] = useState<string[]>([]);
  const [partials, setPartials] = useState<string[]>([]);

  const [rendered, setRendered] = useState<Rendered>({
    seed: "anagram",
    sentence: ["agar", "man"],
  });

  const [executionTime, setExecutionTime] = useState(0);

  useEffect(() => {
    generate({ seed: "anagram", minLength: 3 });
  }, []);

  const generate = ({
    seed,
    minLength,
  }: {
    seed: string;
    minLength: number;
  }) => {
    setLoading(true);
    init().then(() => {
      const start = window.performance.now();
      const { anagrams, partials } = js_generate(
        seed.toLowerCase(),
        minLength || 5
      );
      setResults([...anagrams]);
      setPartials(
        [...partials].sort((a, b) => {
          return b.length - a.length;
        })
      );
      setRenderedSeed(seed);
      const end = window.performance.now();
      setExecutionTime(Math.floor(end - start));
      setLoading(false);
    });
  };

  return (
    <div className="flex w-full h-screen max-w-screen-md p-0 mx-auto md:p-4">
      <div className="flex w-full border border-black">
        <div className="flex flex-col w-1/2 border-r border-black xs:w-1/2 sm:w-1/3">
          <Input loading={loading} generate={generate} />

          <div className="px-2 pt-1 pb-1 text-sm text-gray-400 border-b border-black">
            {loading ? (
              <>Loading...</>
            ) : (
              <>
                {results.length.toLocaleString("en-US")} results in{" "}
                {executionTime.toLocaleString("en-US")}ms
              </>
            )}
          </div>

          {!loading && (
            <Tab.Group as={Fragment}>
              <Tab.List className="flex text-sm border-b border-black">
                <Tab as={Fragment}>
                  {({ selected }) => (
                    <button
                      className={`${
                        selected ? "bg-black text-white" : "hover:bg-slate-100"
                      } w-1/2 py-1 border-r border-black outline-none`}
                    >
                      Partials
                    </button>
                  )}
                </Tab>
                <Tab as={Fragment}>
                  {({ selected }) => (
                    <button
                      className={`${
                        selected ? "bg-black text-white" : "hover:bg-slate-100"
                      } w-1/2 py-1 outline-none`}
                    >
                      Partitions
                    </button>
                  )}
                </Tab>
              </Tab.List>
              <Tab.Panels as={Fragment}>
                <Tab.Panel as={Fragment}>
                  <Results
                    results={partials}
                    setRendered={setRendered}
                    renderedSeed={renderedSeed}
                  />
                </Tab.Panel>
                <Tab.Panel as={Fragment}>
                  <Results
                    results={results}
                    setRendered={setRendered}
                    renderedSeed={renderedSeed}
                  />
                </Tab.Panel>
              </Tab.Panels>
            </Tab.Group>
          )}
        </div>
        <div className="flex flex-col items-center justify-center mx-auto">
          {rendered !== null && (
            <>
              <Poem seed={rendered.seed} sentence={rendered.sentence} />
            </>
          )}
        </div>
      </div>
    </div>
  );
};
