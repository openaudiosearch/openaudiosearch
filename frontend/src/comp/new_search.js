import React, { Component } from 'react';
import { DataSearch, ResultList, CategorySearch, ReactiveBase, ReactiveList} from '@appbaseio/reactivesearch';
import { Flex, Stack, Box, Text, Heading, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert } from '@chakra-ui/core'


const { ResultListWrapper } = ReactiveList;


class SearchPage2 extends Component {
    render() {
        return (
            <ReactiveBase
                app="oas_feed2"
                url="http://localhost:9200"
            >
                <Heading mb='2'>Search</Heading>
                <DataSearch
                componentId="headline/description"
                dataField={["headline", "description"]}
                title="Search"
                fieldWeights={[3, 1]}
                placeholder="Search for feeds"
                autosuggest={true}
                highlight={true}
                highlightField="headline"
                queryFormat="or"
                fuzziness={1}
                />
                
            </ReactiveBase>
        );
    }
}

function Results ( props ) { 
return (
<ReactiveList
    // dataField="dateModified"
    componentId="SearchResult"
    // pagination
    react={{
        "and": 'headline/description'
    }}
>
    {({ data, error, loading, ... }) => (
        <ResultListWrapper>
            {
                data.map(item => (
                    <ResultList key={item.identifier}>
                        <ResultList.Content>
                            <ResultList.Title
                                dangerouslySetInnerHTML={{
                                    __html: item.headline
                                }}
                            />
                            <ResultList.Description>
                                <div>
                                    <div>von {item.creator}</div>
                                </div>
                                <span>
                                    Pub {item.datePublished}
                                </span>
                            </ResultList.Description>
                        </ResultList.Content>
                    </ResultList>
                ))
            }
        </ResultListWrapper>
    )}
</ReactiveList>
)}

export default SearchPage2