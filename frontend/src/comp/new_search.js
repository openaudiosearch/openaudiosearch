import React, { Component } from 'react';
import { DataSearch, ResultList, CategorySearch, ReactiveBase, ReactiveList} from '@appbaseio/reactivesearch';
import { Flex, Stack, Box, Text, Heading, IconButton, Input, Button, useDisclosure, Link, FormControl, Select, FormLabel, Spinner, AlertIcon, Alert } from '@chakra-ui/react'


const { ResultListWrapper } = ReactiveList;


class SearchPage2 extends Component {
    render() {
        return (
            <ReactiveBase
                app="oas_feed2"
                url="http://localhost:9200"
            >
                <Heading mb='2'>Search now</Heading>
                <DataSearch
                    componentId="headline/description"
                    dataField={["headline", "description"]}
                    title="Search"
                    fieldWeights={[5, 1]}
                    placeholder="Search for feeds"
                    autosuggest
                    highlight
                    highlightField="headline"
                    queryFormat="and"
                    fuzziness={0}
                />
                <ReactiveList
                    // dataField="dateModified"
                    componentId="SearchResults"
                    // pagination
                    react={{
                        "and": 'headline/description'
                    }}
                >
                    {({ data, error, loading, ...args }) => (
                        <ResultListWrapper>
                            {
                                data.map(item => (
                                    <ResultList key={item.identifier}>
                                        <ResultList.Content>
                                            {/* <ResultList.Image src={item.image} /> */}
                                            <ResultList.Title
                                                dangerouslySetInnerHTML={{
                                                    __html: item.headline
                                                }}
                                            />
                                            <ResultList.Description>
                                                <div>
                                                    <div>von {item.creator}</div>
                                                    <div>{item.publisher}</div>
                                                </div>
                                                <span>
                                                    gesendet am: {item.datePublished}
                                                </span>
                                            </ResultList.Description>
                                        </ResultList.Content>
                                    </ResultList>
                                ))
                            }
                        </ResultListWrapper>
                    )}
                </ReactiveList>
            </ReactiveBase>
        );
    }
}


export default SearchPage2
